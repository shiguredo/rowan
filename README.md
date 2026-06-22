# shiguredo_rowan

[![crates.io](https://img.shields.io/crates/v/shiguredo_rowan.svg)](https://crates.io/crates/shiguredo_rowan)
[![docs.rs](https://docs.rs/shiguredo_rowan/badge.svg)](https://docs.rs/shiguredo_rowan)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![GitHub Actions](https://github.com/shiguredo/rowan/actions/workflows/ci.yaml/badge.svg)](https://github.com/shiguredo/rowan/actions/workflows/ci.yaml)
[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?logo=discord&logoColor=white)](https://discord.gg/shiguredo)

## 概要

時雨堂による [rust-analyzer/rowan](https://github.com/rust-analyzer/rowan) のフォークです。

以下の変更を加えてあります。

- パッケージ名を `shiguredo_rowan` に変更
- `xtask` を削除し、 `prek.toml` を追加
- CI を `cargo run -p xtask -- ci` から `cargo fmt --all -- --check` と `cargo clippy --all-targets` と `cargo test` と `cargo test --benches --no-run` に変更
- `Cargo.toml` から `[workspace]` の `xtask` メンバーを削除
- Rust の MSRV を 1.85.0 から 1.88.0 に更新
- 外部依存ライブラリを削減（ `countme` 、 `serde` 、 `m_lexer` を削除）
- `hashbrown` を 0.15.2 から 0.17.1 に更新
- `rustc-hash` を 2.1.1 から 2.1.2 に更新
- criterion による wall-clock ベンチマーク基盤を追加（ `benches/green` 、 `benches/cursor` 、 `benches/api` ）
- examples の clippy 警告を修正
