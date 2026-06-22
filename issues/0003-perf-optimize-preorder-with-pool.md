# preorder 走査の allocation を pool allocation で削減する

- Priority: Medium
- Created: 2026-06-22
- Completed: YYYY-MM-DD
- Model: Kimi Code CLI
- Branch: feature/perf-optimize-preorder-with-pool
- Polished: YYYY-MM-DD

## 目的

大きな構文木の preorder 走査時に発生する `NodeData` の allocation 回数を減らし、速度を向上させる。

## 優先度根拠

preorder 走査はパーサーや言語サーバーで頻繁に使われる。木のサイズに比例した allocation は大きなファイルや深い木で顕著になり、速度とメモリ効率に影響する。本家 `rust-analyzer/rowan` でも PR #121 で pool allocation による最適化がドラフト提案されている。一方で、速度・安全性・シンプルさを最優先する方針のもと、無理な実装は避ける。

## 現状

- `src/cursor.rs` の `preorder()` / `preorder_with_tokens()` は、木のサイズに比例して `NodeData` を allocate する。
- 原則として同時に生きているノードは木の高さ分だけなので、pool allocator を使えば多くの allocation を回避できる可能性がある。
- 本家 PR #121 は `src/pool.rs` を新規追加し、`src/cursor.rs` と `src/arc.rs` を変更している。

## 設計方針

- 本家 PR #121 を参考に、専用の pool allocator を検討する。
- ただし、速度・安全性・シンプルさを最優先し、本家 PR をそのまま採用するかは慎重に判断する。
- mutable tree との整合性を保つ。特に `NodeData` の参照カウントと SLL（sorted linked list）の不変条件を崩さないようにする。
- pool allocator が不要な場合やリスクが高い場合は、より小さな最適化（例：スタック上で一定数を確保する）から始める。

## 完了条件

- 大きな木の preorder 走査において、allocation 回数が削減される。
- 既存の `preorder()` / `preorder_with_tokens()` API は維持される。
- ベンチマークで速度向上または allocation 削減が確認される。
- すべてのテストが通ること。
