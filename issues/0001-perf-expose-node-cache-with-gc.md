# NodeCache を公開し GC 能力を追加してメモリ効率を向上させる

- Priority: High
- Created: 2026-06-22
- Completed: YYYY-MM-DD
- Model: GLM-5.2
- Branch: shiguredo
- Polished: 2026-06-22

## 目的

LALRPOP など bottom-up なパーサーでも `NodeCache` を直接利用できるようにし、green tree の構造共有によるメモリ削減を有効にする。さらに、長時間動作するプロセスで `NodeCache` が無限に増大するのを防ぐ GC 能力を追加する。

## 優先度根拠

本家 `rust-analyzer/rowan` でも `NodeCache` の公開を求める声（issue #53、issue #177）があり、PR #63 として提案されている。

## 現状

- `NodeCache` 構造体は `pub` かつ `#[derive(Default, Debug)]` であり、`NodeCache::default()` で外部から作成可能。`GreenNodeBuilder::with_cache(cache: &mut NodeCache)` も `pub` で、キャッシュの外部作成・共有・再利用は既に可能。実際に `benches/common/input.rs` の `build_tree_with_cache` と `tests/bench_fixtures.rs` で利用されている。
- `NodeCache::node()` / `NodeCache::token()` は `pub(crate)` であり、クレート外部から直接呼び出せない。これらが `GreenNodeBuilder` 経由でのみ利用可能な制限となっている。
- `node()` のシグネチャは `fn node(&mut self, kind: SyntaxKind, children: &mut Vec<(u64, GreenElement)>, first_child: usize) -> (u64, GreenNode)` であり、引数に `GreenElement`（`pub(super)` 型エイリアス）、`u64`（private 関数 `node_hash` / `token_hash` で計算される事前ハッシュ）、`first_child: usize`（builder 内部状態のインデックス）を含む。そのまま公開するには内部型の露出が必要であり、bottom-up パーサーにとっても使いやすい API ではない。
- `GreenNode` / `GreenToken` は `ThinArc`（参照カウント式 Arc）であり、`Arc::is_unique()`（`src/arc.rs:154`、`pub(crate)`）で「参照カウント == 1」を判定できる。キャッシュが1つ参照を持っているため、live tree から参照されていないエントリは `is_unique() == true` で特定できる。ただし `GreenNode` / `GreenToken` に `is_unique()` を呼ぶ経路が現状ない。
- `node()` は子数 > 3 のノードを intern しない（`src/green/node_cache.rs:81`）。
- GC 能力は一切存在しない。`NodeCache` はエントリを蓄積し続けるのみで、削除・クリア・LRU 等のメソッドはない。

## 設計方針

### NodeCache の公開化

- bottom-up パーサーが求める「子要素からノードを構築し intern する」API を新設する。
- 新規 public API のシグネチャ:
  - `pub fn intern_node(&mut self, kind: SyntaxKind, children: impl IntoIterator<Item = GreenElement>) -> GreenNode`
  - `pub fn intern_token(&mut self, kind: SyntaxKind, text: &str) -> GreenToken`
  - `impl IntoIterator` で十分。`intern_node` 内部で `Vec` に `collect` し、既存の `node()` に渡す。`node()` 内部の `drain().map()` は `ExactSizeIterator` を実装しているため、`GreenNode::new` の `ExactSizeIterator` 要件は `node()` 内部で満たされる。
- **呼び出し契約**: `intern_node` に渡す子要素は `intern_node` / `intern_token` で intern 済みのものを渡すこと。非 intern 子（`GreenNode::new` で直接構築したもの等）を渡した場合、ポインタ等価比較が常に失敗しキャッシュミスになるだけでなく、毎回新規エントリが挿入されキャッシュが膨張する。正確性には影響しないが構造共有の恩恵が得られず、キャッシュ汚染が発生する。長時間の使用では `gc()` の定期的な呼び出しを推奨する。この契約を doc comment に記載する。
- **実装方式**: 既存の `pub(crate) fn node()` を再利用する。`intern_node` 内で `children` から `Vec<(u64, GreenElement)>` を構築し（各子のハッシュを `node_hash` / `token_hash` で計算）、`node(kind, &mut vec, 0).1` を返す。`node()` / `token()` は `pub(crate)` のまま維持する。
- **ハッシュ再計算コスト**: `intern_node` は子のハッシュを毎回 `node_hash` / `token_hash` で計算する。`node_hash` は再帰的に子孫を走査するため、深さ d の木では全体で O(d * n) になる。一方 `GreenNodeBuilder` は `self.children` に事前計算ハッシュを保持し O(n) で済む。この差は bottom-up パーサーが子から順に intern するユースケースでは許容範囲と判断する。
- **hash==0 センチネル挙動の違い**: `node()` は子のハッシュが 0 の場合（子数 > 3 等）、親の intern もスキップする（`src/green/node_cache.rs:90-93`）。`GreenNodeBuilder` は子の `node()` が返したハッシュ 0 を `self.children` に保持し、祖先に向かって伝播させる。一方 `intern_node` は子のハッシュを `node_hash()` で再計算するため、子数 > 3 の子でも非零のハッシュを返す。したがって `intern_node` と `GreenNodeBuilder` を混在使用した場合、ハッシュ値が異なりキャッシュ lookup に差が出る場合がある。この挙動差を doc comment に記載する。
- `GreenElement` 型エイリアスを `pub` にする。以下の 3 箇所を変更する:
  - `src/green/element.rs`: `pub(super) type GreenElement` → `pub type GreenElement`
  - `src/green.rs`: `use self::element::GreenElement;` → `pub use self::element::GreenElement;`
  - `src/lib.rs`: `pub use crate::green::{...}` に `GreenElement` を追加
- **`GreenElement` の `pub` 化の副作用**: `GreenNode::new` / `replace_child` / `insert_child` / `splice_children`（いずれも `src/green/node.rs`）の引数型として `GreenElement` が使われている。現状は `pub(super)` のため型エイリアス名を参照できないが、`pub` 化により型エイリアス名で参照できるようになる。これは API surface の拡張であり、`CHANGES.md` に記載する。

### GC 能力の追加

- 参照カウント方式（`is_unique()` ベース）を採用する。キャッシュのみが参照しているエントリ（`is_unique() == true`、すなわち live tree から参照されていない）を削除する。シンプルで安全、外部依存追加なし。
- GC メソッドのシグネチャ: `pub fn gc(&mut self)`。引数なし、戻り値なし。
- **実装方式**: `hashbrown::HashMap::retain` で `is_unique()` なエントリを削除する。処理順序は先に `self.nodes` を処理し、後に `self.tokens` を処理する。削除されたノードの `Drop` で子 token の参照カウントが下がるため、`tokens` 側の判定がより正確になる。
- `GreenNode` / `GreenToken` に `pub(crate) fn is_unique(&self) -> bool` を追加する。`ThinArc::with_arc` 経由で `Arc::is_unique()` を呼ぶ実装とする。
- **複数パスの必要性**: `retain` のイテレーション順序により、親が削除される前に子が評価された場合、子の参照カウントがまだ下がっておらず `is_unique() == false` となり残る場合がある。1 回の `gc()` で全ての不要エントリが削除されるとは限らない。複数回 `gc()` を呼ぶことで推移的に不要なエントリを完全に削除できる。この挙動を doc comment に記載する。
- **GC 後の intern 不変式**: GC で子エントリが削除されても親エントリが残る場合、「親は intern 済みだが子は intern されていない」状態が発生する。この状態で同じ構造のノードを再 intern すると、子が新しいポインタで intern され、親のポインタ等価比較が失敗する。正確性には影響しないが、キャッシュ効率が低下する。この挙動を doc comment に記載する。
- **呼び出し制約**: `GreenNodeBuilder::with_cache(cache: &mut NodeCache)` でキャッシュを借用中（ビルダーが生存中）は、借用チェッカーにより `gc()` を呼べない。`gc()` はビルダーを使い終えた後に呼ぶ必要がある。この制約を doc comment に記載する。
- LRU 方式や総メモリ量ベースの GC は、アクセス時刻の追跡やサイズ計算が必要で複雑性が高いため、本 issue では採用しない。

### 変更対象ファイル

- `src/green/node_cache.rs` — `intern_node` / `intern_token` / `gc` メソッドの追加、および新規に `#[cfg(test)] mod tests` を作成してポインタ等価性の検証テストを追加
- `src/green/node.rs` — `GreenNode::is_unique()` の追加（`pub(crate)`）
- `src/green/token.rs` — `GreenToken::is_unique()` の追加（`pub(crate)`）
- `src/green/element.rs` — `GreenElement` 型エイリアスの `pub` 化
- `src/green.rs` — `GreenElement` の re-export を `pub use` に変更
- `src/lib.rs` — `GreenElement` を `pub use` に追加
- `tests/node_cache.rs` — 新規追加。構造等価性・GC の動作・混在使用の統合テスト
- `benches/common/input.rs` — `build_tree_with_intern`（`intern_node` / `intern_token` 経由の bottom-up 構築ヘルパー）を追加
- `benches/green.rs` — `intern_node` / `intern_token` 経由のベンチマークを追加
- `CHANGES.md` — 変更履歴の記載（`[ADD]` 種別で `intern_node` / `intern_token` / `gc` / `GreenElement` の公開をまとめる）

### 本家 PR #63 との関係

本家 PR #63 は `strong_count()` ベースの GC（`strong_count() <= 2` で削除判定）を実装しているが、fork の `src/arc.rs` には `strong_count()` が存在せず `is_unique()`（count == 1 判定）のみである。本 issue は `is_unique()` 方式を採用し、`strong_count()` の追加は行わない。`is_unique()` 方式は「キャッシュのみが参照している」エントリのみを削除するため、PR #63 より保守的だが安全。

## 完了条件

- `NodeCache::intern_node` / `NodeCache::intern_token` が `pub` で追加され、外部から `GreenNodeBuilder` を経由せずに intern できる。
- `GreenElement` 型エイリアスが `pub` になり、外部から `NodeOrToken<GreenNode, GreenToken>` として構築・参照できる。
- `NodeCache::gc` が `pub` で追加され、live tree から参照されていないエントリを削除できる。
- GC 後もキャッシュの整合性が保たれ、引き続き intern に使用できる。
- 既存の `GreenNodeBuilder` API（`new` / `with_cache` / `token` / `start_node` / `finish_node` / `checkpoint` / `start_node_at` / `finish`）と互換性が保たれる。
- 追加する public API（`intern_node` / `intern_token` / `gc`）に doc comment を記載する。特に `intern_node` の子の intern 義務・hash==0 センチネル挙動の違い、`gc` の複数パス必要性・呼び出し制約を記載する。
- `src/green/node_cache.rs` の `#[cfg(test)] mod tests` に以下のポインタ等価性の検証テストを追加する（`pub(crate)` にアクセスできるため）:
  - 同じ `NodeCache` で複数回 `intern_node` を呼ぶと同一ポインタが返ることの検証
  - 子数 > 3 のノードは intern されず、2 回目の `intern_node` で別ポインタが返ることの検証
  - `intern_token` で同一 kind + text に同じポインタが返ることの検証
  - `intern_node` と `GreenNodeBuilder` を同じ `NodeCache` で混在使用し（子数 ≤ 3 の構造で）、構造共有されることの検証
- `tests/node_cache.rs` に以下の統合テストを追加する:
  - `intern_node` / `intern_token` で構築した木が `GreenNodeBuilder` 経由と等価になることの検証
  - `intern_node` の子に `GreenToken` を含めた構築ができることの検証
  - 非 intern 子を渡した場合、キャッシュミスになるが正確性は保たれることの検証
  - `gc` 呼出後に live tree から参照されているエントリが残ることの検証
  - `gc` 呼出後に live tree から参照されていないエントリが削除されることの検証
  - `gc` 呼出後に引き続き intern に使用できることの検証
  - `gc` を複数回呼ぶことで推移的に不要なエントリが削除されることの検証
  - 混在使用後に `gc()` を呼び、参照カウント管理が正しく機能することの検証
  - 空キャッシュに対する `gc` の検証
- `benches/green.rs` に `intern_node` / `intern_token` 経由の構築ベンチマークを追加し、`build_tree_with_cache`（`GreenNodeBuilder::with_cache` 経由）との性能比較を行う。ハッシュ再計算コストの影響を確認する。
- `CHANGES.md` に変更履歴を記載する（`shiguredo-changelog` スキル参照）。
- `cargo fmt --all -- --check` が通る。
- `RUSTFLAGS="-D warnings" cargo clippy --all-targets` が通る。
- `cargo test` が通る（既存テスト + 追加テスト）。
- `cargo test --test tidy` が通る。
- `cargo test --benches --no-run` が通る。
- `cargo doc --no-deps` が warning なしで通る。
- `cargo publish --dry-run` が通る。
