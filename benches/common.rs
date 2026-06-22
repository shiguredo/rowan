//! ベンチマーク共通モジュール
//!
//! # --save-baseline の運用方法
//!
//! criterion の `--save-baseline <name>` を使うと、計測結果を名前付きベースラインとして保存できる。
//! 保存したベースラインは `--baseline <name>` で比較対象に指定できる。
//!
//! ## ベースライン名の命名
//!
//! 変更前後で比較できるよう、ベースライン名には変更の前後と識別子を含めること。
//!
//! - `before-<日付>` : 変更前の計測に使う（例 : `before-20260622` ）
//! - `after-<日付>` : 変更後の計測に使う（例 : `after-20260622` ）
//!
//! ## 比較方法
//!
//! 1. 変更前のコードで `cargo bench -- --save-baseline before-20260622` を実行し、ベースラインを保存する
//! 2. 変更後のコードで `cargo bench -- --baseline before-20260622` を実行し、保存したベースラインと比較する
//!
//! `--baseline <name>` を指定すると、 criterion は保存したベースラインとの差分を表示する。
//! 変更後の計測結果も保存したい場合は `--save-baseline after-20260622 --baseline before-20260622` のように併用する。

pub mod input;
pub mod lang;
