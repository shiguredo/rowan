# examples の clippy 警告を修正する

- Priority: Low
- Created: 2026-06-22
- Completed: YYYY-MM-DD
- Model: opencode-go/glm-5.2
- Branch: feature/fix-examples-clippy-warnings
- Polished: YYYY-MM-DD

## 目的

`examples/math.rs` と `examples/s_expressions.rs` の clippy 警告を修正し、 `RUSTFLAGS="-D warnings" cargo clippy --all-targets` が通るようにする。

## 優先度根拠

現状の CI は `cargo test` と `cargo fmt -- --check` のみ実行し clippy は実行しないたち、直ちに CI が失敗するわけではない。しかし `shiguredo-rust` スキルが `#[expect]` の使用などを規約で定めており、 clippy 警告を放置すると「 Don't live with broken windows 」に違反する。将来的に CI に clippy を追加した際に障害になるため、 Low 優先度で対応する。

## 現状

`RUSTFLAGS="-D warnings" cargo clippy --all-targets` を実行すると、 examples で以下の clippy 警告が発生する。

### `examples/math.rs`

- `upper_case_acronyms` : `WHITESPACE` / `ADD` / `SUB` / `MUL` / `DIV` / `NUMBER` / `ERROR` / `OPERATION` / `ROOT` の各列挙子名が大文字の頭字語を含む（ 9 件）

### `examples/s_expressions.rs`

- `upper_case_acronyms` : `WORD` / `WHITESPACE` / `ERROR` / `LIST` / `ATOM` / `ROOT` の各列挙子名が大文字の頭字語を含む（ 6 件）
- `empty_line_after_doc_comments` : 194 行目の doc コメントの後に空行があり、 196 行目の type alias をドキュメントしていない（ 1 件）

両ファイルとも `#[allow(non_camel_case_types)]` で抑制しているが、 `upper_case_acronyms` は別 lint のため抑制できていない。

## 設計方針

- `upper_case_acronyms` への対応は、列挙子名を clippy の推奨どおり先頭以外小文字に変更する（例 : `WHITESPACE` → `Whitespace` 、 `ADD` → `Add` ）。ただし examples は公開 API ではないため、 `#[allow(clippy::upper_case_acronyms)]` で抑制する選択も妥当。どちらを採用するかは実装時に判断する。
- `empty_line_after_doc_comments` は空行を削除するか、 doc コメントを通常コメント（ `//` ）に変更する。
- `shiguredo-rust` スキルの規約に従い、 lint 抑制は `#[allow]` ではなく `#[expect]` を使う（ただし examples は `#[allow(non_camel_case_types)]` が既に使われているため、既存方針との整合も考慮する）。

## 完了条件

- `RUSTFLAGS="-D warnings" cargo clippy --all-targets` が通る。
- `cargo test` が通る。
- `cargo fmt --check` が通る。
- 既存のテストがすべて通る。
