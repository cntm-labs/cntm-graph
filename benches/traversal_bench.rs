use cntm_graph::GraphStore;
use criterion::{Criterion, criterion_group, criterion_main};
use std::fs;

fn scalar_traversal(store: &GraphStore, target_type: u16) -> (usize, f32) {
    let type_slice = store.nodes.get_type_slice();
    let weight_slice = store.nodes.get_weight_slice();

    let mut best_idx = 0;
    let mut best_score = -1.0;

    for i in 0..store.nodes.count {
        if type_slice[i] == target_type && weight_slice[i] > best_score {
            best_score = weight_slice[i];
            best_idx = i;
        }
    }
    (best_idx, best_score)
}

fn criterion_benchmark(c: &mut Criterion) {
    let path = "bench_graph.bin";
    let _ = fs::remove_file(path);

    let node_count = 1_000_000;
    let mut store = GraphStore::new(path, node_count, 10).unwrap();

    for i in 0..node_count {
        let type_id = if i % 100 == 0 { 42 } else { 1 };
        let weight = (i as f32) * 0.0001;
        store.nodes.add_node(i as u64, type_id, weight);
    }

    let mut group = c.benchmark_group("Traversal");

    group.bench_function("Scalar 1M", |b| b.iter(|| scalar_traversal(&store, 42)));

    group.bench_function("SIMD 1M", |b| b.iter(|| store.find_best_weighted_simd(42)));

    group.finish();
    let _ = fs::remove_file(path);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
