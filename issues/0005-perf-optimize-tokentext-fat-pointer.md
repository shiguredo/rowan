# TokenText を fat pointer 化して indirection を削減する

- Priority: Low
- Created: 2026-06-22
- Completed: YYYY-MM-DD
- Model: Kimi Code CLI
- Branch: feature/perf-optimize-tokentext-fat-pointer
- Polished: YYYY-MM-DD

## 目的

`GreenToken` のテキスト参照時の間接参照を減らし、速度を向上させる。

## 優先度根拠

本家 `rust-analyzer/rowan` の PR #100 では、`Nonnull<str>` を使った fat pointer 化が提案された。しかし、本家では議論の末に採用されておらず、実装の複雑さと安全性のトレードオフが大きい。速度を最優先する一方、安全性とシンプルさも同等に重視する方針に基づき、低優先度で検討する。

## 現状

- `src/green/token.rs` の `GreenToken` は `ThinArc<GreenTokenHead, u8>` でテキストを保持している。
- テキスト参照時、`GreenTokenData::text()` は `self.data.slice()` を経由しており、header から slice への間接参照が入る。
- 本家 PR #100 は `GreenTokenData` 内で `str` への fat pointer を直接持つことで、1 段階の indirection を削減しようとしている。

## 設計方針

- 本家 PR #100 を参考に、`TokenText` 型の導入または `GreenTokenData` 内の fat pointer 化を検討する。
- ただし、本家で採用されなかった理由（実装複雑さ、安全性、メモリレイアウトの不安定性）を踏まえ、無理な導入は避ける。
- 影響が小さく、安全性が担保できる範囲から実験的に始める。
- 速度向上が実測できない場合は採用しない。

## 完了条件

- 速度向上または間接参照削減が確認される。
- 安全性とシンプルさを損なわない。
- 既存の `GreenToken` / `GreenTokenData` API は維持される。
- すべてのテストが通ること。
