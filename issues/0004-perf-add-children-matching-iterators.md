# kind に一致しない子を allocate しない children_matching / siblings_matching を追加する

- Priority: Medium
- Created: 2026-06-22
- Completed: YYYY-MM-DD
- Model: Kimi Code CLI
- Branch: feature/perf-add-children-matching-iterators
- Polished: YYYY-MM-DD

## 目的

特定の `SyntaxKind` を持つ子や兄弟を探す際、一致しない子を allocate しない専用イテレーターを追加し、allocation 回数を削減する。

## 優先度根拠

本家 `rust-analyzer/rowan` の PR #119 では、`children_matching` を使うことで rust-analyzer の completion benchmark で total blocks allocated が 8.5% 減少、total bytes allocated が 4% 減少している。fork 内でも `first_child_by_kind` や `by_kind` イテレーターは存在するが、一致しない子を完全にスキップする最適化は未実装である。速度を最優先する方針に基づき、効果が大きい本改善を検討する。

## 現状

- `src/cursor.rs` には `SyntaxNodeChildren::by_kind` や `SyntaxElementChildren::by_kind` が存在する。
- これらは `first_child_by_kind` / `next_sibling_by_kind` を使っており、一致しない子は `SyntaxNode` として作成された後にフィルタリングされる可能性がある。
- 本家 PR #119 は、一致しない子を `NodeData` として allocate する前に緑木レベルでスキップするアプローチを提案している。

## 設計方針

- `SyntaxNodeChildren` / `SyntaxElementChildren` に `matching` 系メソッドを追加するか、既存の `by_kind` を allocation 削減方向で改善する。
- 一致しない子の `NodeData` 作成を回避する実装を検討する。
- 既存の `by_kind` API と重複しないよう、名前・挙動・用途を明確に分ける。
- 速度と安全性を損なわない範囲で実装する。

## 完了条件

- kind に一致しない子を allocate しないイテレーターが追加される。
- ベンチマークまたはテストで allocation 削減が確認される。
- 既存 API と重複しない命名・設計になっている。
- すべてのテストが通ること。
