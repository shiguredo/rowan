# Rowan

[![docs.rs](https://docs.rs/rowan/badge.svg)](https://docs.rs/rowan/)
[![Crates.io](https://img.shields.io/crates/v/rowan.svg)](https://crates.io/crates/rowan)
[![Crates.io](https://img.shields.io/crates/d/rowan.svg)](https://crates.io/crates/rowan)

Rowan is a library for lossless syntax trees, inspired in part by
Swift's [libsyntax](https://github.com/apple/swift/tree/5e2c815edfd758f9b1309ce07bfc01c4bc20ec23/lib/Syntax).

A conceptual overview is available in the [rust-analyzer book](https://rust-analyzer.github.io/book/contributing/syntax.html).

See `examples/s_expressions` for a tutorial, and [rust-analyzer](https://github.com/rust-analyzer/rust-analyzer/) for real-world usage.

## Testing

This crate is primarily tested by various integration tests in rust-analyzer.

## License

Rowan is primarily distributed under the terms of both the MIT
license and the Apache License (Version 2.0).

See LICENSE-APACHE and LICENSE-MIT for details.

---

時雨堂による rust-analyzer/rowan のフォークです。

以下の変更を加えてあります。

- `xtask` を削除し、`prek.toml` を追加
- CI を `cargo run -p xtask -- ci` から `cargo test` と `cargo fmt -- --check` に変更
- `Cargo.toml` から `[workspace]` の `xtask` メンバーを削除
- Rust の MSRV を 1.85.0 から 1.88.0 に更新
- 外部依存ライブラリを削減（`countme`、`serde`、`m_lexer` を削除）
