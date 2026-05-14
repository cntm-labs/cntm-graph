use cntm_graph::GraphStore;
use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let path = "stress_test_1b.bin";
    let node_count: usize = 1_000_000_000; // 1 Billion
    let edge_count: usize = 100; // Minimal edges for this test

    println!("🚀 Starting Phase 3 Stress Test: 1 Billion Nodes");
    println!("📂 Target File: {}", path);

    let start = Instant::now();
    // Note: This will attempt to create a ~40GB+ file.
    let mut store = GraphStore::new(path, node_count, edge_count)?;
    println!("✅ Allocation completed in {:?}", start.elapsed());

    println!("⚡ Filling 1B nodes with data...");
    let fill_start = Instant::now();
    for i in 0..node_count {
        // Simple data pattern to verify integrity later
        store.add_node(i as u64, (i % 100) as u16, (i as f32) / 1000.0);

        if i > 0 && i % 100_000_000 == 0 {
            println!(
                "... Progress: {}M nodes ({:?})",
                i / 1_000_000,
                fill_start.elapsed()
            );
        }
    }
    println!("✅ Fill completed in {:?}", fill_start.elapsed());

    println!("🔍 Starting Random Access Probe (1M samples)...");
    let mut rng = SmallRng::from_entropy();
    let probe_count = 1_000_000;
    let mut latencies = Vec::with_capacity(probe_count);

    for _ in 0..probe_count {
        let idx = rng.gen_range(0..node_count);
        let sample_start = Instant::now();

        // Access Node Data (Hot Path via raw pointers)
        let _id = unsafe { *store.nodes.ids_ptr.add(idx) };
        let _weight = unsafe { *store.nodes.weights_ptr.add(idx) };

        latencies.push(sample_start.elapsed().as_nanos());
    }

    latencies.sort_unstable();
    let p50 = latencies[probe_count / 2];
    let p99 = latencies[(probe_count as f32 * 0.99) as usize];
    let mean: u128 = latencies.iter().sum::<u128>() / probe_count as u128;

    println!("📊 Results:");
    println!("   - Mean Latency: {} ns", mean);
    println!("   - P50 Latency:  {} ns", p50);
    println!("   - P99 Latency:  {} ns", p99);

    Ok(())
}
