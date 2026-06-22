//! ベンチマーク用 fixture 生成ヘルパー
//!
//! `benches/common/lang.rs` には依存しない。これにより `tests/bench_fixtures.rs` から
//! `#[path = "../benches/common/input.rs"]` で参照しても module tree が破綻しない。

use shiguredo_rowan::{GreenNode, GreenNodeBuilder, NodeCache, SyntaxKind};

/// 内部ノードの kind
pub const NODE: SyntaxKind = SyntaxKind(0);

/// 葉トークンの kind
pub const TOKEN: SyntaxKind = SyntaxKind(1);

/// api ベンチで `first_child_by_kind` の最悪ケース（最後の子に一致）を計測するための kind 。
/// ルートの最後の子にのみ割り当てられる。
pub const TARGET: SyntaxKind = SyntaxKind(2);

/// green / cursor ベンチの計測パラメータ : （深さ、子数、トークン長）
///
/// トークン長は 1 で統一する。全 8 組み合わせ。
//
// 複数のベンチマークターゲットで共有されるモジュールのため、ターゲットによっては未使用になる。
// #[expect] はターゲットごとに lint の発火有無が異なるため使えず、 #[allow] を使用する。
#[allow(dead_code)]
pub const BENCH_PARAMS: &[(usize, usize, usize)] = &[
    (15, 1, 1),
    (15, 2, 1),
    (10, 2, 1),
    (7, 5, 1),
    (5, 10, 1),
    (4, 10, 1),
    (3, 20, 1),
    (2, 50, 1),
];

/// api ベンチの計測パラメータ : （深さ、子数、トークン長）
///
/// api ベンチは c >= 2 かつ d >= 3 の組み合わせのみ使用する。
/// c = 1 は子が 1 つのみで `first_child_by_kind` の最悪ケースにならない。
/// d = 2 は子がすべてトークンで `children()` が空になるため。
//
// 複数のベンチマークターゲットで共有されるモジュールのため、ターゲットによっては未使用になる。
// #[expect] はターゲットごとに lint の発火有無が異なるため使えず、 #[allow] を使用する。
#[allow(dead_code)]
pub const API_BENCH_PARAMS: &[(usize, usize, usize)] =
    &[(15, 2, 1), (10, 2, 1), (7, 5, 1), (5, 10, 1), (4, 10, 1), (3, 20, 1)];

/// 深さ `depth` 、子数 `children` 、トークン長 `token_len` の n-ary tree を構築する。
///
/// 深さはルートを 1 と数える。各内部ノードは `children` 個の子を持ち、
/// 深さ `depth` の葉は長さ `token_len` のトークンとなる。
///
/// ルートの最後の子を `TARGET` 、他の内部ノードを `NODE` 、葉トークンを `TOKEN` として構築する。
/// `TARGET` ノードの部分木は通常の n-ary tree と同じ構造とする。
///
/// # 前提
///
/// `depth >= 2` であること。深さ 1 は前提外（ `build_subtree` が無限再帰して
/// スタックオーバーフローするため）。
///
/// # 総要素数
///
/// - `children == 1` の場合 : `depth` （深さ 1 〜 depth-1 がノード、深さ depth がトークン）
/// - `children != 1` の場合 : `(children^depth - 1) / (children - 1)`
pub fn build_tree(depth: usize, children: usize, token_len: usize) -> GreenNode {
    // トークン長は全トークンで共通のため、 1 回だけ確保して使い回す
    let text = "a".repeat(token_len);
    let mut builder = GreenNodeBuilder::new();
    build_subtree(&mut builder, depth, children, &text, 1, NODE);
    builder.finish()
}

/// 指定された `NodeCache` を使って n-ary tree を構築する。
///
/// `build_tree` と同じ構造の木を生成するが、キャッシュを再利用する。
/// キャッシュを再利用することで、構造的に等しい部分木が共有され、メモリ使用量と構築コストが削減される。
//
// 複数のベンチマークターゲットで共有されるモジュールのため、ターゲットによっては未使用になる。
// #[expect] はターゲットごとに lint の発火有無が異なるため使えず、 #[allow] を使用する。
#[allow(dead_code)]
pub fn build_tree_with_cache(
    cache: &mut NodeCache,
    depth: usize,
    children: usize,
    token_len: usize,
) -> GreenNode {
    // トークン長は全トークンで共通のため、 1 回だけ確保して使い回す
    let text = "a".repeat(token_len);
    let mut builder = GreenNodeBuilder::with_cache(cache);
    build_subtree(&mut builder, depth, children, &text, 1, NODE);
    builder.finish()
}

/// n-ary tree の部分木を再帰的に構築する。
///
/// `current_depth` は現在のノードの深さ（ルートを 1 とする）。
/// `kind` は現在のノードの kind 。
/// `text` は葉トークンのテキスト（全トークンで共通）。
///
/// 子が葉トークンになる深さ（ `current_depth + 1 == depth` ）までは内部ノードを再帰的に構築し、
/// 葉の一つ手前の深さで `children` 個のトークンを追加する。
/// ルート（ `current_depth == 1` ）の最後の子のみ `TARGET` とし、
/// それ以外の内部ノードは `NODE` とする。
fn build_subtree(
    builder: &mut GreenNodeBuilder<'_>,
    depth: usize,
    children: usize,
    text: &str,
    current_depth: usize,
    kind: SyntaxKind,
) {
    builder.start_node(kind);
    if current_depth + 1 == depth {
        // 子は葉トークン
        for _ in 0..children {
            builder.token(TOKEN, text);
        }
    } else {
        // 子は内部ノード
        for i in 0..children {
            // ルートの最後の子を TARGET にする
            let child_kind = if current_depth == 1 && i + 1 == children { TARGET } else { NODE };
            build_subtree(builder, depth, children, text, current_depth + 1, child_kind);
        }
    }
    builder.finish_node();
}
