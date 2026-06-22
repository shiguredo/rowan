# 既存依存のバージョン指定と用途コメントを整理する

- Priority: Low
- Created: 2026-06-22
- Completed: YYYY-MM-DD
- Model: opencode-go/glm-5.2
- Branch: feature/refactor-dependency-version-comments
- Polished: YYYY-MM-DD

## 目的

`Cargo.toml` の既存依存（ `hashbrown` / `rustc-hash` / `text-size` ）のバージョン指定を `shiguredo-rust` スキルの規約（マイナーバージョンまで）に合わせ、用途コメントを追加する。

## 優先度根拠

`shiguredo-rust` スキルが「バージョン番号はマイナーバージョンまで指定すること」「依存ライブラリには用途をコメントで明記すること」を規約で定めている。現状の既存依存は両規約に違反している。直ちに動作に影響はないが、規約違反の放置は「 Don't live with broken windows 」に反するため Low 優先度で対応する。

## 現状

`Cargo.toml` の `[dependencies]` セクション :

```toml
hashbrown = {
  version = "0.17.1",
  default-features = false,
  features = [
    "inline-more",
    "raw-entry",
  ]
}
rustc-hash = "2.1.2"
text-size = "1.1.1"
```

問題 :

- `hashbrown` の `version` が `"0.17.1"` とパッチまで指定されている（規約では `"0.17"` ）
- `rustc-hash` のバージョンが `"2.1.2"` とパッチまで指定されている（規約では `"2.1"` ）
- `text-size` のバージョンが `"1.1.1"` とパッチまで指定されている（規約では `"1.1"` ）
- 3 つの依存すべてに用途コメントがない

なお issue 0006 で追加した `criterion = { version = "0.8", ... }` は規約に準拠しており、用途コメントも付いている。

## 設計方針

- バージョン指定をマイナーバージョンまでに変更する。
  - `hashbrown` : `"0.17.1"` → `"0.17"`
  - `rustc-hash` : `"2.1.2"` → `"2.1"`
  - `text-size` : `"1.1.1"` → `"1.1"`
- 各依存に用途コメントを追加する。
  - `hashbrown` : green ノード・トークンの interning 用 HashMap として使用
  - `rustc-hash` : `FxHasher` 用（ `NodeCache` のハッシュ計算）
  - `text-size` : `TextSize` / `TextRange` 用（オフセット管理）
- issue 0006 で追加した `criterion` の用途コメントのスタイルに合わせる。

## 完了条件

- `Cargo.toml` の既存 3 依存のバージョン指定がマイナーバージョンまでになる。
- 各依存に用途コメントが追加される。
- `cargo check --all-targets` が通る。
- `cargo test` が通る。
- `cargo fmt --check` が通る。
- `cargo publish --dry-run` が通る。
