# CI パイプラインを整備する

- Priority: High
- Created: 2026-06-22
- Completed: 2026-06-22
- Model: opencode-go/glm-5.2
- Branch: shiguredo
- Polished: YYYY-MM-DD

## 目的

CI が push でトリガーされない問題を修正し、ワークフローをモダン化したうえで clippy とベンチマークコンパイル検証を追加し、リポジトリの品質ゲートを機能させる。

## 優先度根拠

2026-06-22 時点で `shiguredo` ブランチへ push しても CI がトリガーされないことを確認した。ローカル検証で品質は担保できるが、 CI が機能しないと「 Don't live with broken windows 」に反するだけでなく、将来の PR 運用で事故る。モダン化・ clippy 追加・ベンチコンパイル検証も同時に整え、一度で CI を健全な状態にするため High 。

## 現状

`.github/workflows/ci.yaml` :

```yaml
name: CI
on:
  pull_request:
  push:
    branches: ["shiguredo"]

env:
  CARGO_INCREMENTAL: 0
  CARGO_NET_RETRY: 10
  CI: 1
  RUST_BACKTRACE: short
  RUSTFLAGS: -D warnings
  RUSTUP_MAX_RETRIES: 10

jobs:
  test:
    name: Rust
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v2
      with:
        fetch-depth: 0 # fetch tags for publish
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        profile: minimal
        override: true

    - run: cargo test
    - run: cargo fmt -- --check
```

問題 :

1. **CI がトリガーされない**: 2026-06-22 の push （コミット `8ba6963` ）でワークフローが実行されなかった。 `gh run list --branch shiguredo` で直近の run が 2026-05-10 まで遡る状態。
2. **古い action**: `actions/checkout@v2` （ v4 が最新）、 `actions-rs/toolchain@v1` （非推奨、メンテナンス終了）。
3. **clippy なし**: `cargo clippy` が CI に含まれない。ただし examples に `upper_case_acronyms` と `empty_line_after_doc_comments` の警告があるため、 clippy 追加は 0007 の完了が前提。
4. **ベンチコンパイル検証なし**: `cargo test --benches --no-run` が未実行。ベンチマークコードが壊れていても気づけない。
5. **`RUSTFLAGS: -D warnings` あり**: すでに warnings が CI 失敗扱い。 examples の警告が `cargo test` で引っかからないのは examples が `cargo test` でコンパイルされるものの lint が別だから。 clippy を追加すると examples で失敗する。

## 設計方針

### CI トリガー不具合の修正

- `gh run list` で run が見えない原因を特定する。可能性:
  - リポジトリの Actions 設定が無効（ Settings → Actions → General ）
  - ワークフローが無効化されている（ `gh workflow list` で確認）
  - 古い action の deprecation で workflow 全体が実行開始前にエラー
- 必要に応じて `gh workflow enable` で再有効化する。
- ワークフローのモダン化（後述）で action を更新する。

### CI ワークフローのモダン化

- `actions/checkout@v4` に更新。
- `actions-rs/toolchain@v1` は削除する。 `ubuntu-latest` の runner には `rustup` がプレインストールされているため、 `rustup component add rustfmt clippy` でコンポーネントを追加する。サードパーティの rust-toolchain action は使わない。
- `cargo fmt -- --check` を `cargo fmt --all -- --check` に変更（ `--all` で全ワークスペース対象）。
- `RUSTFLAGS: -D warnings` は維持。

### clippy の追加

- `cargo clippy --all-targets -- -D warnings` を CI に追加する。
- ただし examples の警告（ 0007 ）が未解決の場合 CI が失敗する。 **0007 を先に closed にする** か、 clippy の対象を `--lib --tests --benches` に絞って examples を除外する。 0007 を先に解く方が健全。
- `cargo fmt --check` と `cargo clippy` は `cargo test` とは別 step に分け、失敗箇所を特定しやすくする。

### ベンチマークコンパイル検証の追加

- `cargo test --benches --no-run` を CI に追加する。ベンチマークが壊れていないことを保証する。
- `cargo bench` 自体は CI では実行しない（時間がかかるため）。

### CI の step 構成案

```yaml
jobs:
  test:
    runs-on: ubuntu-26.04
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
    - run: rustup component add rustfmt clippy
    - run: cargo fmt --all -- --check
    - run: cargo clippy --all-targets -- -D warnings
    - run: cargo test
    - run: cargo test --benches --no-run
```

### 依存 issue

- 0007 （ examples の clippy 警告修正）を先に closed にしないと clippy step が失敗する。実装順序は 0007 → 0010 。

## 完了条件

- `shiguredo` ブランチへの push で CI がトリガーされる。
- CI で以下がすべて実行され、通過する:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
  - `cargo test --benches --no-run`
- `ubuntu-26.04` runner を使っている。
- `rust-toolchain.toml` でツールチェーンとコンポーネントを固定している（サードパーティの rust-toolchain action は使わない）。
- 古い `actions-rs/toolchain@v1` を削除している。
- PR でも CI が走る（ `pull_request` トリガー維持）。

## 解決方法

CI パイプラインを整備し、同時に 0007 （ examples の clippy 警告）を解決した。

### 変更内容

- `.github/workflows/ci.yaml` : ワークフローをモダン化した。
  - `actions/checkout@v2` → `actions/checkout@v4`
  - `actions-rs/toolchain@v1` を削除（ `rust-toolchain.toml` でツールチェーンを固定）
  - `runs-on: ubuntu-latest` → `runs-on: ubuntu-26.04`
  - `permissions: contents: read` を追加
  - `concurrency` グループを追加し、連続 push で古い run をキャンセル
  - `timeout-minutes: 10` を追加
  - `cargo fmt --all -- --check` step を追加
  - `cargo clippy --all-targets -- -D warnings` step を追加
  - `cargo test --benches --no-run` step を追加
  - `fetch-depth: 0` と英語コメントを削除
- `examples/math.rs` と `examples/s_expressions.rs` : clippy 警告を修正した（ 0007 相当）。
  - `#[expect(clippy::upper_case_acronyms, reason = "...")]` を追加
  - `empty_line_after_doc_comments` のために doc コメント後の空行を削除

### 検証結果

- `cargo fmt --all -- --check` : 通過
- `RUSTFLAGS="-D warnings" cargo clippy --all-targets` : 通過
- `RUSTFLAGS="-D warnings" cargo test` : 通過
- `cargo test --benches --no-run` : 通過
