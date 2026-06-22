# Cargo.toml の include 見直しで publish 時の warning を解消する

- Priority: Low
- Created: 2026-06-22
- Completed: YYYY-MM-DD
- Model: opencode-go/glm-5.2
- Branch: feature/fix-cargo-include-list
- Polished: YYYY-MM-DD

## 目的

`cargo publish --dry-run` で examples と tests が `include` に含まれていないという warning が発生する問題を解消する。

## 優先度根拠

`cargo publish --dry-run` を実行すると以下の warning が出る。 publish 本番で warning が出る状態は好ましくなく、 `shiguredo-rust` スキルの「依存は最小限にすること」やクリーンなパッケージングの観点からも整理すべき。ただし publish されたクレートの機能には影響しないたち Low 優先度とする。

## 現状

`Cargo.toml` の `include` :

```toml
include = ["/LICENSE", "/README.md", "/src/**/*", "/benches/**/*.rs"]
```

`cargo publish --dry-run` の warning :

```
warning: ignoring example `math` as `examples/math.rs` is not included in the published package
warning: ignoring example `s_expressions` as `examples/s_expressions.rs` is not included in the published package
warning: ignoring test `bench_fixtures` as `tests/bench_fixtures.rs` is not included in the published package
warning: ignoring test `tidy` as `tests/tidy.rs` is not included in the published package
```

examples と tests が `include` に含まれていないため、 publish 時にこれらが無視される。 crates.io からインストールしたユーザーは examples を参照できず、 `cargo test` も実行できない。

## 設計方針

以下のいずれかの方向性で対応する。実装時にどちらが適切か判断する。

### 选项 A : examples と tests を include に追加する

```toml
include = ["/LICENSE", "/README.md", "/src/**/*", "/benches/**/*.rs", "/examples/**/*.rs", "/tests/**/*.rs"]
```

publish されたクレートでも examples と tests を参照・実行できる。

### 选项 B : examples と tests を publish 対象外とし、 warning を抑制する

`include` は現状のまま残し、 `cargo publish` 時の warning を受け入れる。ただし warning は残るため、 `Cargo.toml` に `publish = true` を維持しつつ warning を出さない方法はない。 选项 A が推奨。

### 推奨

选项 A を推奨する。 examples と tests はクレートの利用価値を高める要素であり、 publish 対象に含めるべき。

## 完了条件

- `cargo publish --dry-run` で examples と tests に関する warning が出ない。
- `cargo test` が通る。
- `cargo fmt --check` が通る。
- `cargo publish --dry-run` が通る。
