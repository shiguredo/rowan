//! `api::SyntaxNode` の子ノード走査のベンチマーク

mod common;

use std::hint::black_box;

use common::{
    input,
    lang::{BenchKind, BenchLang},
};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use shiguredo_rowan::api::SyntaxNode;

/// `children()` 計測 : 全子ノードを消費する
fn children(c: &mut Criterion) {
    let mut group = c.benchmark_group("api");
    for &(depth, children, token_len) in input::API_BENCH_PARAMS {
        // 木は b.iter 外で構築する
        let green = input::build_tree(depth, children, token_len);
        let node = SyntaxNode::<BenchLang>::new_root(green);
        let id = format!("d{depth}_c{children}_t{token_len}_children");
        group.bench_with_input(BenchmarkId::from_parameter(id), &node, |b, node| {
            b.iter(|| {
                for child in node.children() {
                    black_box(child);
                }
            });
        });
    }
    group.finish();
}

/// 最初の一致探索計測 : 最悪ケース（最後の子が Target ）
fn first_child_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("api");
    for &(depth, children, token_len) in input::API_BENCH_PARAMS {
        // 木は b.iter 外で構築する
        let green = input::build_tree(depth, children, token_len);
        let node = SyntaxNode::<BenchLang>::new_root(green);
        let id = format!("d{depth}_c{children}_t{token_len}_first_child_matching");
        group.bench_with_input(BenchmarkId::from_parameter(id), &node, |b, node| {
            b.iter(|| {
                let found = node.children().find(|child| child.kind() == BenchKind::Target);
                black_box(found);
            });
        });
    }
    group.finish();
}

/// `children().filter` 計測 : Target に一致する子を全消費する
fn children_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("api");
    for &(depth, children, token_len) in input::API_BENCH_PARAMS {
        // 木は b.iter 外で構築する
        let green = input::build_tree(depth, children, token_len);
        let node = SyntaxNode::<BenchLang>::new_root(green);
        let id = format!("d{depth}_c{children}_t{token_len}_children_matching");
        group.bench_with_input(BenchmarkId::from_parameter(id), &node, |b, node| {
            b.iter(|| {
                node.children().filter(|child| child.kind() == BenchKind::Target).for_each(|n| {
                    black_box(n);
                });
            });
        });
    }
    group.finish();
}

criterion_group!(benches, children, first_child_matching, children_matching);
criterion_main!(benches);
