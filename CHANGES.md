# Changelog

## develop

- [CHANGE] upstream 0.17.0 に追従し typed mutable API と cursor mutation engine を削除する
  - @voluntas
- [CHANGE] `SyntaxNode::green` が借用した `GreenNodeData` を返すように変更する
  - @voluntas
- [CHANGE] CI パイプラインをモダン化し clippy とベンチマークコンパイル検証を追加する
  - @voluntas
- [ADD] `tree_top` メソッドを追加する
  - @voluntas
- [ADD] criterion による wall-clock ベンチマーク基盤を追加する
  - @voluntas
- [FIX] examples の clippy 警告を修正する
  - @voluntas
