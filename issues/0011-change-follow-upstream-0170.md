# upstream の 0.17.0 に追従して mutable API を削除する

- Priority: High
- Created: 2026-08-05
- Completed: YYYY-MM-DD
- Model: opencode/deepseek-v4-flash
- Branch: feature/change-follow-upstream-0170
- Polished: YYYY-MM-DD

## 目的

upstream の `rust-analyzer/rowan` が master で 0.17.0 に上がったため、フォークを追従させる。 0.17.0 は typed mutable API と cursor mutation engine を削除する後方互換のない変更を含んでおり、フォークの公開 API にも同じ破壊的変更を適用する。

## 現状

- upstream は master を v0.15.15 起点にリセットして再構築しており、新 master (0.17.0) には v0.15.16 〜 v0.16.1 系のコミットが含まれない。旧 master 系は `v0.16` ブランチに残存している。
- フォークは旧 master (v0.15.16 系) をベースにしており、`src/api.rs` と `src/cursor.rs` に mutable API (`new_root_mut` / `splice_children` / `insert_children` / `set_text` / Mutation) を公開している。 0.17.0 ではこれらが削除される (`replace_with` は残る)。
- 0.17.0 では `src/sll.rs` と SLL ベースの兄弟ノード最適化も削除されているため、`src/cursor.rs` の `to_next_sibling` 系の unsound 最適化 (issue 0002) は upstream 側の削除で解消される。
- upstream の master リセットにより、v0.16 系で修正済みだった問題 (prev_sibling の off-by-one など) が新 master に含まれていない可能性がある。

## 設計方針

- `src/` を upstream 0.17.0 の内容に置き換え、フォーク固有の差分を再適用する。
  - 依存削減: `countme` / `serde` / `memoffset` を除去する (フォークの依存削減方針を維持する)
  - doc 内のクレート名を `shiguredo_rowan` に書き換える
- フォークの非コードファイル (`Cargo.toml` / CI / README / benches / examples / prek / clippy / rust-toolchain) はフォークの設定を維持する。
- 新 master で失われた修正 (prev_sibling の off-by-one など) が新 master にも存在するバグの場合は再適用する。
- 0002 は upstream 側の削除で解消された扱いで closed にする。

## 完了条件

- `cargo test` が通る。
- `cargo clippy --all-targets` が警告なしで通る。
- `cargo bench --no-run` が通る。
- `README.md` と `CHANGES.md` が 0.17.0 追従の内容に更新されている。
- 0002 が closed に移動されている。
