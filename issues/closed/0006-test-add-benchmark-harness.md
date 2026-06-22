# criterion による wall-clock ベンチマーク基盤を追加する

- Priority: Medium
- Created: 2026-06-22
- Completed: 2026-06-22
- Model: Kimi Code CLI
- Branch: shiguredo
- Polished: 2026-06-22

## 目的

性能改善の効果を客観的に測定し、無駄な最適化を避ける。

## 優先度根拠

計測なしでは「効果がありそう」という推測に依存し、実装してから効果がなければ削除コストがかかる。計測基盤を先に整備することで、#0001 や #0004 など効果が期待できる改善を優先し、#0003 や #0005 など効果が不明瞭な改善を測定後に判断できる。#0002 の速度影響測定にも必要となる。

## 現状

- 本プロジェクトにはベンチマークが存在しない。
- `Cargo.toml` に `criterion` が含まれていない。
- `Cargo.toml` に `[[bench]]` ターゲットが存在しないため、`cargo bench` で criterion の `--save-baseline` などのオプションが使えない。

## 設計方針

- `Cargo.toml` に `[dev-dependencies]` セクションを新規追加し、`criterion = { version = "0.8", default-features = false, features = ["cargo_bench_support"] }` を追加する。用途コメントを併記する。criterion 0.8 の MSRV は 1.86 であり、プロジェクトの `rust-version = "1.88.0"` と整合する。`plotters` や `rayon` は wall-clock 計測に必須ではないため default features は無効にする。これにより HTML レポートは生成されず、テキスト出力のみになる。
- `Cargo.toml` の `[package]` に `autobenches = false` を追加する。
- `Cargo.toml` に以下の `[lib]` セクションを追加する。`cargo bench` 実行時に lib target がベンチマークターゲットとしてコンパイル・実行されるのを防ぐためである。
  ```toml
  [lib]
  path = "src/lib.rs"
  bench = false
  ```
- `Cargo.toml` に以下の `[[bench]]` セクションを追加する。
  ```toml
  [[bench]]
  name = "green"
  harness = false

  [[bench]]
  name = "cursor"
  harness = false

  [[bench]]
  name = "api"
  harness = false
  ```
- `Cargo.toml` の `include` に `"/benches/**/*.rs"` を追加する。`[[bench]]` エントリを追加した状態で `cargo package` / `cargo publish` を通すためである。
- `benches/` ディレクトリを以下の構成で追加する。プロジェクトのモジュールスタイル（`src/green.rs` + `src/green/node.rs`）に合わせて `mod.rs` は使わず `common.rs` とする。
  - `benches/common.rs` — `pub mod input; pub mod lang;` を宣言する
  - `benches/common/input.rs` — 入力 fixture 生成ヘルパー
  - `benches/common/lang.rs` — api layer 用のベンチマーク専用最小 `Language` 実装
  - `benches/green.rs` — `GreenNodeBuilder` による構文木構築
  - `benches/cursor.rs` — `cursor::SyntaxNode::preorder` / `preorder_with_tokens`
  - `benches/api.rs` — `api::SyntaxNode::children` / `first_child_by_kind` / `children().by_kind(...)`
- 各ベンチファイルの先頭に `mod common;` を宣言する。各ベンチファイルの `main()` 関数は `criterion_group!` / `criterion_main!` マクロで生成する。
- `benches/common/input.rs` では、深さ `d`、子数 `c`、トークン長 `t` をパラメータとして持つ n-ary tree をプログラム生成する。深さはルートを 1 と数える。各内部ノードは `c` 個の子を持ち、深さ `d` の葉は長さ `t` のトークンとする。`c ≠ 1` の場合、総要素数は `(c^d - 1) / (c - 1)` となる。`c = 1` の場合、総要素数は `d` となる（深さ 1 〜 d-1 がノード、深さ d がトークン）。トークンのテキストは `"a".repeat(token_len)` とする。
- `benches/common/input.rs` は以下の関数を提供する。パラメータの型はすべて `usize` とする。
  - `pub fn build_tree(depth: usize, children: usize, token_len: usize) -> GreenNode` — `GreenNodeBuilder` を使って `GreenNode` を構築する。
  - `pub fn build_tree_with_cache(cache: &mut NodeCache, depth: usize, children: usize, token_len: usize) -> GreenNode` — 指定された `NodeCache` を使って `GreenNode` を構築する。
  - 内部で使用する raw `SyntaxKind` 定数 `NODE`, `TOKEN`, `TARGET` を公開する。`NODE = SyntaxKind(0)`, `TOKEN = SyntaxKind(1)`, `TARGET = SyntaxKind(2)` である。
- `build_tree` はルートの最後の子を `TARGET`、他の内部ノードを `NODE`、葉トークンを `TOKEN` として構築する。TARGET ノードの部分木は通常の n-ary tree と同じ構造とする。これにより api ベンチで `first_child_by_kind` の最悪ケース（最後の子に一致）を計測できる。green/cursor ベンチでは kind は走査性能に影響しないため、同じ fixture を共用する。
- `benches/common/input.rs` は `benches/common/lang.rs` には依存させない。これにより `tests/bench_fixtures.rs` から `#[path = "../benches/common/input.rs"]` で参照しても module tree が破綻しない。`input.rs` 内では `shiguredo_rowan::...` など絶対クレート名を使用する。
- `benches/common/lang.rs` では、ベンチマーク専用の最小 `Language` を定義する。
  - `#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] struct BenchLang;`
  - `#[repr(u16)] #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] enum BenchKind { Node = 0, Token = 1, Target = 2 }`
  - `impl shiguredo_rowan::Language for BenchLang { ... }`
  - `kind_from_raw` は `match` で safe に変換する。`raw.0` が 0〜2 以外の場合は `panic!` で安全に失敗させる。`kind_to_raw` は `SyntaxKind(kind as u16)` で変換する。
- 計測パラメータは以下の組み合わせを採用する。括弧内は総要素数である。トークン長は `t = 1` で統一する。
  - 深さ 15 / 子数 1: 15
  - 深さ 15 / 子数 2: 32,767
  - 深さ 10 / 子数 2: 1,023
  - 深さ 7 / 子数 5: 19,531
  - 深さ 5 / 子数 10: 11,111
  - 深さ 4 / 子数 10: 1,111
  - 深さ 3 / 子数 20: 421
  - 深さ 2 / 子数 50: 51
- 各パラメータと benchmark の対応は以下のとおりとする。
  - green / cursor ベンチ: 全 8 組み合わせを使用する。
  - api ベンチ: c ≥ 2 かつ d ≥ 3 の組み合わせのみ使用する（c = 1 は子が 1 つのみで `first_child_by_kind` の最悪ケースにならない、d = 2 は子がすべてトークンで `children()` が空になるため）。
- 子数 1 は鎖状の木を生成し、走査系の再帰・反復性能を測定する。子数 2 は `NodeCache` の効果を測定するのに適する。子数 5 以上は #0004 の children matching iterators の効果を測定するのに適する。
- criterion の `BenchmarkId` は `d{depth}_c{children}_t{token_len}` とし、グループ名は `green`、`cursor`、`api` とする。`benches/green.rs` の fresh cache／reuse cache は `BenchmarkId` に `fresh_cache` / `reuse_cache` の接尾辞を付けて区別する。
- 各ベンチマークでは、測定対象ループ内では `std::hint::black_box` を使用して最適化を抑制する。
  - `benches/green.rs`: fixture 構築に必要なパラメータを `b.iter` 外で決定する。fresh cache 計測では `b.iter` 内で `input::build_tree(...)` を呼び出す。`GreenNodeBuilder::new()` は内部で `NodeCache` を生成するが、イテレーションごとに新規作成されるためキャッシュは再利用されない。reuse cache 計測では `b.iter` 外で `NodeCache` を 1 つ生成し、`b.iter` 内で `input::build_tree_with_cache(&mut cache, ...)` を呼び出す。reuse cache 計測は warm cache 状態を測定する（初回 iteration は cold cache だが、criterion の多数 iteration により warm が支配的になる）。cache は各ベンチマーク関数ごとに独立させ、ベンチマーク間では共有しない。fresh cache と reuse cache は別の benchmark 関数とし、`BenchmarkId` の接尾辞で区別する。
  - `benches/cursor.rs`: `input::build_tree(...)` で `GreenNode` を `b.iter` 外で構築し、`shiguredo_rowan::cursor::SyntaxNode` に変換する。`preorder()` と `preorder_with_tokens()` は別の benchmark 関数とし、`BenchmarkId` の接尾辞 `preorder` / `preorder_with_tokens` で区別する。`b.iter` 内で `for event in tree.preorder() { std::hint::black_box(event); }` で全文走査する。
  - `benches/api.rs`: `input::build_tree(...)` で `GreenNode` を `b.iter` 外で構築し、`shiguredo_rowan::api::SyntaxNode<BenchLang>` に変換する。`children()` / `first_child_by_kind` / `children().by_kind` は別の benchmark 関数とし、`BenchmarkId` の接尾辞 `children` / `first_child_by_kind` / `children_by_kind` で区別する。root ノードを対象として、`b.iter` 内で `for child in node.children() { std::hint::black_box(child); }` で `children()` を全消費し、`first_child_by_kind(|kind| kind == BenchKind::Target)` を呼び出して結果を `std::hint::black_box` に渡し、`node.children().by_kind(|kind| kind == BenchKind::Target).for_each(|n| std::hint::black_box(n))` で `children().by_kind(...)` を全消費する。
- criterion の計測設定はデフォルトを使用する。
- wall-clock time の計測に限定する。
- fixture の正当性を保証するため、`tests/bench_fixtures.rs` を追加する。`tests/bench_fixtures.rs` からは `#[path = "../benches/common/input.rs"]` で `benches/common/input.rs` を、`#[path = "../benches/common/lang.rs"]` で `benches/common/lang.rs` を参照し、以下を検証する。
  - 総要素数（ノード + トークン）が期待値と一致すること。期待値は入力パラメータから計算する関数を用いる。c ≠ 1 の場合は `(c^d - 1) / (c - 1)`、c = 1 の場合は `d` とする。
  - ルートの子数が `c` と一致すること（c ≥ 1 の場合）。
  - 総テキスト長が「トークン数 × トークン長」と一致すること。
  - 期待値計算関数が fixture 生成と同じバグを共有するリスクを避けるため、一部の代表的なパラメータ（深さ 15 / 子数 2 = 32767、深さ 5 / 子数 10 = 11111 など）は hardcoded 値でも検証する。
  - 境界値パラメータとして子数 1 とトークン長 0 を含める。深さ 1 は `build_tree` の前提外（ルートがトークンになるため `GreenNodeBuilder::finish()` が panic する）とし、テストでは深さ ≥ 2 を前提とする。
  - `BenchLang` の `Language` 実装のラウンドトリップテスト（`kind_from_raw(kind_to_raw(k)) == k` for all `k in [Node, Token, Target]`）を含める。

## 完了条件

- `cargo check --all-targets` が通る。
- `cargo test --benches --no-run` でベンチマークコードがコンパイル・リンクされる。
- `cargo bench -- --test` でベンチマークが smoke 実行できる。
- `cargo test` で `tests/bench_fixtures.rs` が通る。
- `cargo clippy --all-targets` が通る。
- `cargo fmt --check` が通る。
- `cargo test --test tidy` が通る。
- 既存のテストがすべて通る。
- `cargo publish --dry-run` が通る。
- 追加された dev-dependencies および間接依存が `rust-version` で `cargo check --all-targets` を通過する。
- ベンチマークが `cargo bench` で実行できる。
- `cargo bench -- --save-baseline <name>` が実行可能であることを確認する。
- `benches/common.rs` に `--save-baseline` の運用方法（ベースライン名の命名、比較方法）をコメントで記載する。

## 解決方法

criterion 0.8 を用いた wall-clock ベンチマーク基盤を追加した。

### 変更内容

- `Cargo.toml` : `autobenches = false` 、 `[dev-dependencies]` に `criterion` 、 `[lib]` セクション、 `[[bench]]` セクション（ green / cursor / api ）、 `include` に `/benches/**/*.rs` を追加した。
- `benches/common.rs` : `--save-baseline` の運用方法（ベースライン名の命名、比較方法）をコメントで記載した。
- `benches/common/input.rs` : fixture 生成ヘルパー（ `build_tree` / `build_tree_with_cache` / `BENCH_PARAMS` / `API_BENCH_PARAMS` / `NODE` / `TOKEN` / `TARGET` 定数）を追加した。
- `benches/common/lang.rs` : ベンチマーク専用の最小 `Language` 実装（ `BenchLang` / `BenchKind` ）を追加した。
- `benches/green.rs` : `GreenNodeBuilder` による構文木構築のベンチマーク（ fresh cache / reuse cache ）を追加した。
- `benches/cursor.rs` : `cursor::SyntaxNode` の preorder 走査のベンチマーク（ preorder / preorder_with_tokens ）を追加した。
- `benches/api.rs` : `api::SyntaxNode` の子ノード走査のベンチマーク（ children / first_child_by_kind / children_by_kind ）を追加した。
- `tests/bench_fixtures.rs` : fixture の正当性を検証するテスト（ 16 件）を追加した。総要素数・トークン数・テキスト長・ルート子数・hardcoded 期待値・境界値・kind 構造（子数 1 / 2 / 50 ）・ `BenchLang` のラウンドトリップ・ `kind_from_raw` の panic を検証する。
- `src/green/builder.rs` : doctest の `use rowan::` を `use shiguredo_rowan::` に修正した。
- `src/green/node.rs` : `green_siblings` の lifetime を明示した。
- `src/cursor.rs` : `splice_children` のループを `zip` に整理し、 lifetime を明示した。
- `src/syntax_text.rs` : `found` 関数を `res.err()` に簡略化した。

### 検証結果

- `cargo check --all-targets` : 通過
- `cargo test --benches --no-run` : 通過
- `cargo bench -- --test` : 通過（全 smoke 実行 Success ）
- `cargo test` : 通過（ bench_fixtures 16 件 + 既存テスト全通過）
- `cargo clippy --all-targets` : 通過
- `cargo fmt --check` : 通過
- `cargo test --test tidy` : 通過
- `cargo publish --dry-run` : 通過
- `cargo bench -- --save-baseline` : 動作確認済み
