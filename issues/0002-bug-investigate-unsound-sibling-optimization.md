# to_next_sibling_or_token 系の unsound 最適化を調査・修正する

- Priority: High
- Created: 2026-06-22
- Completed: 2026-08-05
- Model: Kimi Code CLI
- Branch: feature/fix-investigate-unsound-sibling-optimization
- Polished: YYYY-MM-DD

## 目的

`cursor.rs` の `to_next_sibling` / `to_next_sibling_or_token` 系メソッドが mutable tree 上で安全に動作することを確認し、問題があれば修正する。

## 優先度根拠

本家 `rust-analyzer/rowan` の issue #172 で、同系の最適化が unsound であることが報告されている。fork 内でも同じパターンの最適化を採用しており、安全性に関わる問題として優先的に調査する必要がある。速度よりも安全性を最優先する方針に基づく。

## 現状

- `src/cursor.rs` の `SyntaxNode::to_next_sibling` および `SyntaxElement::to_next_sibling_or_token` は、`self` の所有権を消費して既存の `NodeData` を書き換える最適化を行っている。
- 本家 issue #172 では、`free` 関数内で assertion failed する panic が報告されており、mutable tree の SLL（sorted linked list）不変条件が崩れることが原因と推測されている。
- 現時点で fork 内で本家と同じ現象が再現するかは未確認。

## 設計方針

- mutable tree と immutable tree の両方で `to_next_sibling` / `to_next_sibling_or_token` の安全性を調査する。
- 必要に応じて、mutable tree では in-place 更新を止め、新規 `NodeData` を allocate する安全な実装に戻す。
- immutable tree でも同じ問題が起こらないことを検証する。
- 速度面への影響をベンチマークで確認し、安全性とのトレードオフを明示する。

## 完了条件

- 本家 #172 と同根の panic が再現しないこと、または再現した場合は修正されること。
- `cursor.rs` 内の該当メソッドに対する安全な実装が完了すること。
- 安全性を担保するテストが追加されること。
- すべてのテストが通ること。

## 解決方法

upstream の 0.17.0 で cursor mutation engine と `src/sll.rs` が削除され、 `src/cursor.rs` の `to_next_sibling` / `to_next_sibling_or_token` メソッドが存在しなくなった。フォークは 0.17.0 に追従したため、unsound の原因となっていた SLL ベースの兄弟ノード最適化はコードベースから完全に消えている。該当コードが存在しないため調査・修正は不要と判断し closed にする。
