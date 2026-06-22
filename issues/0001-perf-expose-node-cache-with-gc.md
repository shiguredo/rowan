# NodeCache を公開し GC 能力を追加してメモリ効率を向上させる

- Priority: High
- Created: 2026-06-22
- Completed: YYYY-MM-DD
- Model: Kimi Code CLI
- Branch: feature/perf-expose-node-cache-with-gc
- Polished: YYYY-MM-DD

## 目的

LALRPOP など bottom-up なパーサーでも `NodeCache` を直接利用できるようにし、green tree の構造共有によるメモリ削減を有効にする。さらに、長時間動作するプロセスで `NodeCache` が無限に増大するのを防ぐ GC 能力を追加する。

## 優先度根拠

本家 `rust-analyzer/rowan` でも `NodeCache` の公開を求める声（issue #53、issue #177）があり、PR #63 として提案されている。fork 内でも `GreenNodeBuilder::with_cache` は存在するが、`NodeCache` を外部で作成・共有できないため機能が不完全である。構造共有は rowan の主要なメモリ効率の源泉であり、これを有効にすることでパーサーのメモリ使用量を大幅に削減できる可能性がある。

## 現状

- `NodeCache` 構造体は `pub` になっているが、`node()` / `token()` メソッドは `pub(crate)` であり、クレート外部から呼び出せない。
- `GreenNodeBuilder::with_cache` は存在するが、キャッシュを外部で作成・共有できないため、実質的に利用されていない。
- `src/green/node_cache.rs` 内で実装されているキャッシュの intern 処理をそのまま公開すればよい。

## 設計方針

- `NodeCache::node` / `NodeCache::token` を public にする。引数・返り値は既存の `pub(crate)` 版と整合させる。
- キャッシュエントリ数や総メモリ量、あるいは LRU 的な基準で GC するメソッドを追加する。
- 速度と安全性を損なわない範囲で実装する。GC はオプショナルな API とし、既存の `GreenNodeBuilder` 利用者に強制しない。
- 本家 PR #63 を参考にするが、fork の現状に合わせて無理な外部依存は追加しない。

## 完了条件

- 外部から `NodeCache` を作成・共有・再利用できる。
- キャッシュを GC できる API が追加される。
- 既存の `GreenNodeBuilder` API と互換性が保たれる。
- テストが追加され、すべてのテストが通る。
