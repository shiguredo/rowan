//! `GreenNodeBuilder` による構文木構築のベンチマーク
//!
//! fresh cache 計測ではイテレーションごとに新規キャッシュで木を構築し、
//! reuse cache 計測では同じ `NodeCache` を再利用して warm cache 状態を計測する。

mod common;

use std::hint::black_box;

use common::input;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use shiguredo_rowan::NodeCache;

/// fresh cache 計測 : イテレーションごとに新規キャッシュで木を構築する
fn fresh_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("green");
    for &(depth, children, token_len) in input::BENCH_PARAMS {
        let id = format!("d{depth}_c{children}_t{token_len}_fresh_cache");
        group.bench_with_input(
            BenchmarkId::from_parameter(id),
            &(depth, children, token_len),
            |b, &(depth, children, token_len)| {
                b.iter(|| {
                    let tree = input::build_tree(depth, children, token_len);
                    black_box(tree);
                });
            },
        );
    }
    group.finish();
}

/// reuse cache 計測 : 同じ `NodeCache` を再利用して木を構築する
fn reuse_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("green");
    for &(depth, children, token_len) in input::BENCH_PARAMS {
        // キャッシュは各パラメータごとに独立させる
        let mut cache = NodeCache::default();
        let id = format!("d{depth}_c{children}_t{token_len}_reuse_cache");
        group.bench_with_input(
            BenchmarkId::from_parameter(id),
            &(depth, children, token_len),
            |b, &(depth, children, token_len)| {
                b.iter(|| {
                    let tree = input::build_tree_with_cache(&mut cache, depth, children, token_len);
                    black_box(tree);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, fresh_cache, reuse_cache);
criterion_main!(benches);
