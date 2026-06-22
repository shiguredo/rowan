//! ベンチマーク fixture の正当性を検証するテスト
//!
//! `benches/common/input.rs` と `benches/common/lang.rs` を `#[path]` で参照し、
//! fixture 生成ロジックが正しい木を構築していることを検証する。

use shiguredo_rowan::{GreenNodeData, Language, NodeCache, NodeOrToken, SyntaxKind};

#[path = "../benches/common/input.rs"]
mod input;

#[path = "../benches/common/lang.rs"]
mod lang;

use input::{build_tree, build_tree_with_cache};
use lang::{BenchKind, BenchLang};

/// 総要素数（ノード + トークン）を再帰的に数える
fn count_elements(node: &GreenNodeData) -> usize {
    1 + node
        .children()
        .map(|child| match child {
            NodeOrToken::Node(n) => count_elements(n),
            NodeOrToken::Token(_) => 1,
        })
        .sum::<usize>()
}

/// トークン数を再帰的に数える
fn count_tokens(node: &GreenNodeData) -> usize {
    node.children()
        .map(|child| match child {
            NodeOrToken::Node(n) => count_tokens(n),
            NodeOrToken::Token(_) => 1,
        })
        .sum::<usize>()
}

/// 期待される総要素数を計算する
///
/// - `children == 1` の場合 : `depth` （深さ 1 〜 depth-1 がノード、深さ depth がトークン）
/// - `children != 1` の場合 : `(children^depth - 1) / (children - 1)`
fn expected_elements(depth: usize, children: usize) -> usize {
    if children == 1 { depth } else { (children.pow(depth as u32) - 1) / (children - 1) }
}

/// 期待されるトークン数を計算する
///
/// - `children == 1` の場合 : 1 （深さ depth のトークン 1 つ）
/// - `children != 1` の場合 : `children^(depth - 1)` （深さ depth の葉の数）
fn expected_tokens(depth: usize, children: usize) -> usize {
    if children == 1 { 1 } else { children.pow((depth - 1) as u32) }
}

#[test]
fn 総要素数が期待値と一致する() {
    for &(depth, children, token_len) in input::BENCH_PARAMS {
        let tree = build_tree(depth, children, token_len);
        let actual = count_elements(&tree);
        let expected = expected_elements(depth, children);
        assert_eq!(
            actual, expected,
            "総要素数が不一致 : depth={depth}, children={children}, token_len={token_len}"
        );
    }
}

#[test]
fn ルートの子数が_children_と一致する() {
    for &(depth, children, token_len) in input::BENCH_PARAMS {
        let tree = build_tree(depth, children, token_len);
        let actual = tree.children().count();
        assert_eq!(
            actual, children,
            "ルートの子数が不一致 : depth={depth}, children={children}, token_len={token_len}"
        );
    }
}

#[test]
fn トークン数が期待値と一致する() {
    // 実際に木を走査してトークン数を数え、公式からの期待値と照合する
    for &(depth, children, token_len) in input::BENCH_PARAMS {
        let tree = build_tree(depth, children, token_len);
        let actual = count_tokens(&tree);
        let expected = expected_tokens(depth, children);
        assert_eq!(
            actual, expected,
            "トークン数が不一致 : depth={depth}, children={children}, token_len={token_len}"
        );
    }
}

#[test]
fn 総テキスト長がトークン数とトークン長の積と一致する() {
    for &(depth, children, token_len) in input::BENCH_PARAMS {
        let tree = build_tree(depth, children, token_len);
        let actual = u32::from(tree.text_len()) as usize;
        let expected = expected_tokens(depth, children) * token_len;
        assert_eq!(
            actual, expected,
            "総テキスト長が不一致 : depth={depth}, children={children}, token_len={token_len}"
        );
    }
}

#[test]
fn 代表的なパラメータの要素数が_hardcoded_値と一致する() {
    // 期待値計算関数が fixture 生成と同じバグを共有するリスクを避けるため、
    // hardcoded 値でも検証する
    let cases: &[(usize, usize, usize, usize)] = &[
        // （深さ、子数、トークン長、期待要素数）
        (15, 2, 1, 32767),
        (10, 2, 1, 1023),
        (7, 5, 1, 19531),
        (5, 10, 1, 11111),
        (4, 10, 1, 1111),
        (3, 20, 1, 421),
        (2, 50, 1, 51),
        (15, 1, 1, 15),
    ];
    for &(depth, children, token_len, expected) in cases {
        let tree = build_tree(depth, children, token_len);
        let actual = count_elements(&tree);
        assert_eq!(
            actual, expected,
            "hardcoded 期待値と不一致 : depth={depth}, children={children}, token_len={token_len}"
        );
    }
}

#[test]
fn 代表的なパラメータのトークン数が_hardcoded_値と一致する() {
    // 期待値計算関数が fixture 生成と同じバグを共有するリスクを避けるため、
    // hardcoded 値でもトークン数を検証する
    let cases: &[(usize, usize, usize, usize)] = &[
        // （深さ、子数、トークン長、期待トークン数）
        (15, 2, 1, 16384),
        (10, 2, 1, 512),
        (7, 5, 1, 15625),
        (5, 10, 1, 10000),
        (4, 10, 1, 1000),
        (3, 20, 1, 400),
        (2, 50, 1, 50),
        (15, 1, 1, 1),
    ];
    for &(depth, children, token_len, expected) in cases {
        let tree = build_tree(depth, children, token_len);
        let actual = count_tokens(&tree);
        assert_eq!(
            actual, expected,
            "hardcoded トークン数と不一致 : depth={depth}, children={children}, token_len={token_len}"
        );
    }
}

#[test]
fn 境界値パラメータの検証() {
    // 子数 1 （鎖状の木）とトークン長 0 の境界値を検証する
    // 深さ >= 2 を前提とする

    // 子数 1 / トークン長 0 / 深さ 5
    let tree = build_tree(5, 1, 0);
    assert_eq!(count_elements(&tree), 5, "子数 1 の鎖状の木は要素数 = 深さ");
    assert_eq!(tree.children().count(), 1, "ルートの子数は 1");
    assert_eq!(u32::from(tree.text_len()) as usize, 0, "トークン長 0 の場合、総テキスト長は 0");

    // 子数 1 / トークン長 1 / 深さ 2 （最小の深さ）
    let tree = build_tree(2, 1, 1);
    assert_eq!(count_elements(&tree), 2, "深さ 2 / 子数 1 は要素数 2");
    assert_eq!(tree.children().count(), 1, "ルートの子数は 1");
    assert_eq!(u32::from(tree.text_len()) as usize, 1, "トークン長 1 の場合、総テキスト長は 1");

    // トークン長 0 / 子数 2 / 深さ 3
    let tree = build_tree(3, 2, 0);
    assert_eq!(count_elements(&tree), 7, "深さ 3 / 子数 2 は要素数 7");
    assert_eq!(tree.children().count(), 2, "ルートの子数は 2");
    assert_eq!(u32::from(tree.text_len()) as usize, 0, "トークン長 0 の場合、総テキスト長は 0");
}

#[test]
fn build_tree_with_cache_が同じ要素数を生成する() {
    let mut cache = NodeCache::default();
    for &(depth, children, token_len) in input::BENCH_PARAMS {
        let tree = build_tree_with_cache(&mut cache, depth, children, token_len);
        let actual = count_elements(&tree);
        let expected = expected_elements(depth, children);
        assert_eq!(
            actual, expected,
            "build_tree_with_cache の要素数が不一致 : depth={depth}, children={children}, token_len={token_len}"
        );
    }
}

#[test]
fn build_tree_with_cache_の境界値を検証する() {
    // build_tree と同じ境界値（トークン長 0 ）を build_tree_with_cache でも検証する
    let mut cache = NodeCache::default();
    let tree = build_tree_with_cache(&mut cache, 5, 1, 0);
    assert_eq!(count_elements(&tree), 5, "子数 1 の鎖状の木は要素数 = 深さ");
    assert_eq!(tree.children().count(), 1, "ルートの子数は 1");
    assert_eq!(u32::from(tree.text_len()) as usize, 0, "トークン長 0 の場合、総テキスト長は 0");

    let mut cache = NodeCache::default();
    let tree = build_tree_with_cache(&mut cache, 3, 2, 0);
    assert_eq!(count_elements(&tree), 7, "深さ 3 / 子数 2 は要素数 7");
    assert_eq!(tree.children().count(), 2, "ルートの子数は 2");
    assert_eq!(u32::from(tree.text_len()) as usize, 0, "トークン長 0 の場合、総テキスト長は 0");
}

#[test]
fn build_tree_と_build_tree_with_cache_が同じ構造を生成する() {
    // キャッシュの有無で木の構造（ kind ・要素数・子数・テキスト長）が完全に一致することを検証する
    // GreenNode の PartialEq は kind と children を再帰的に比較するため、
    // assert_eq! で kind 構造も含めて完全な等価性を検証できる
    for &(depth, children, token_len) in input::BENCH_PARAMS {
        let tree_a = build_tree(depth, children, token_len);
        let mut cache = NodeCache::default();
        let tree_b = build_tree_with_cache(&mut cache, depth, children, token_len);
        assert_eq!(
            tree_a, tree_b,
            "木の構造が不一致 : depth={depth}, children={children}, token_len={token_len}"
        );
    }
}

#[test]
fn fixture_の_kind_構造が子数_2_で仕様どおりであること() {
    // ルートの kind が NODE 、ルートの最後の子の kind が TARGET 、
    // ルートの最初の子の kind が NODE 、葉トークンの kind が TOKEN であることを検証する。
    // これにより api ベンチの first_child_by_kind 最悪ケース計測の妥当性を保証する。
    // 検証しやすさのため子数 2 / 深さ 3 の小さい木を使う。
    let tree = build_tree(3, 2, 1);
    assert_eq!(tree.kind(), input::NODE, "ルートの kind は NODE");

    let children: Vec<_> = tree.children().collect();
    assert_eq!(children.len(), 2, "ルートの子数は 2");
    assert_eq!(children[0].kind(), input::NODE, "ルートの最初の子の kind は NODE");
    assert_eq!(children[1].kind(), input::TARGET, "ルートの最後の子の kind は TARGET");

    // 葉トークンの kind が TOKEN であることを検証する
    // cursor::SyntaxNode には kind() が無いため、 green ノードから直接確認する
    let first_child_green = match tree.children().next().unwrap() {
        NodeOrToken::Node(n) => n,
        NodeOrToken::Token(_) => panic!("最初の子はノードであるべき"),
    };
    let leaf = first_child_green.children().next().expect("子を持つ");
    assert_eq!(leaf.kind(), input::TOKEN, "葉トークンの kind は TOKEN");
}

#[test]
fn fixture_の_kind_構造が子数_1_で仕様どおりであること() {
    // 子数 1 （鎖状）の場合、ルートの唯一の子が「最後の子」になり TARGET が割り当てられる。
    // さらに深い階層のノードは NODE であり、 TARGET はルート階層にのみ出現する。
    let tree = build_tree(4, 1, 1);
    assert_eq!(tree.kind(), input::NODE, "ルートの kind は NODE");

    let children: Vec<_> = tree.children().collect();
    assert_eq!(children.len(), 1, "ルートの子数は 1");
    // 唯一の子 = 最後の子 = TARGET
    assert_eq!(children[0].kind(), input::TARGET, "ルートの唯一の子の kind は TARGET");

    // TARGET の子は NODE であり、 TARGET はルート階層にのみ出現する
    let target_green = match children[0] {
        NodeOrToken::Node(n) => n,
        NodeOrToken::Token(_) => panic!("TARGET はノードであるべき"),
    };
    let target_children: Vec<_> = target_green.children().collect();
    assert_eq!(target_children.len(), 1, "TARGET の子数は 1");
    assert_eq!(
        target_children[0].kind(),
        input::NODE,
        "TARGET の子の kind は NODE （ TARGET はルート階層にのみ出現）"
    );
}

#[test]
fn fixture_の_kind_構造が子数_50_で仕様どおりであること() {
    // 子数 50 の場合、ルートの最後の子（ 49 番目）のみ TARGET で、
    // それ以外（ 0 〜 48 番目）は NODE であることを検証する。
    // 深さ 3 にすることで子が内部ノードになり、 kind の検証ができる。
    let tree = build_tree(3, 50, 1);
    assert_eq!(tree.kind(), input::NODE, "ルートの kind は NODE");

    let children: Vec<_> = tree.children().collect();
    assert_eq!(children.len(), 50, "ルートの子数は 50");
    // 最初の子は NODE
    assert_eq!(children[0].kind(), input::NODE, "ルートの最初の子の kind は NODE");
    // 最後の子は TARGET
    assert_eq!(children[49].kind(), input::TARGET, "ルートの最後の子の kind は TARGET");
}

#[test]
fn bench_lang_の_kind_ラウンドトリップが一致する() {
    for kind in [BenchKind::Node, BenchKind::Token, BenchKind::Target] {
        let raw = BenchLang::kind_to_raw(kind);
        let back = BenchLang::kind_from_raw(raw);
        assert_eq!(back, kind, "kind ラウンドトリップが不一致");
    }
}

#[test]
fn bench_lang_の_raw_値が_input_定数と対応する() {
    // input.rs の NODE, TOKEN, TARGET 定数と lang.rs の BenchKind が
    // 直接比較で一致することを検証する。片方だけ変更してもテストが通るのを防ぐ。
    assert_eq!(BenchLang::kind_to_raw(BenchKind::Node), input::NODE);
    assert_eq!(BenchLang::kind_to_raw(BenchKind::Token), input::TOKEN);
    assert_eq!(BenchLang::kind_to_raw(BenchKind::Target), input::TARGET);
}

#[test]
#[should_panic(expected = "invalid raw SyntaxKind 3 for BenchLang (this is a bug)")]
fn bench_lang_の_kind_from_raw_が不正値で_panic_する() {
    // 0-2 以外の raw 値は panic する（実装バグの表明）
    BenchLang::kind_from_raw(SyntaxKind(3));
}
