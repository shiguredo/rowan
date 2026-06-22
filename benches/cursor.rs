//! `cursor::SyntaxNode` の preorder 走査のベンチマーク

mod common;

use std::hint::black_box;

use common::input;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use shiguredo_rowan::cursor::SyntaxNode;

/// `preorder()` 計測 : ノードのみの preorder 走査
fn preorder(c: &mut Criterion) {
    let mut group = c.benchmark_group("cursor");
    for &(depth, children, token_len) in input::BENCH_PARAMS {
        // 木は b.iter 外で構築する
        let green = input::build_tree(depth, children, token_len);
        let tree = SyntaxNode::new_root(green);
        let id = format!("d{depth}_c{children}_t{token_len}_preorder");
        group.bench_with_input(BenchmarkId::from_parameter(id), &tree, |b, tree| {
            b.iter(|| {
                for event in tree.preorder() {
                    black_box(event);
                }
            });
        });
    }
    group.finish();
}

/// `preorder_with_tokens()` 計測 : ノードとトークンの preorder 走査
fn preorder_with_tokens(c: &mut Criterion) {
    let mut group = c.benchmark_group("cursor");
    for &(depth, children, token_len) in input::BENCH_PARAMS {
        // 木は b.iter 外で構築する
        let green = input::build_tree(depth, children, token_len);
        let tree = SyntaxNode::new_root(green);
        let id = format!("d{depth}_c{children}_t{token_len}_preorder_with_tokens");
        group.bench_with_input(BenchmarkId::from_parameter(id), &tree, |b, tree| {
            b.iter(|| {
                for event in tree.preorder_with_tokens() {
                    black_box(event);
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, preorder, preorder_with_tokens);
criterion_main!(benches);
