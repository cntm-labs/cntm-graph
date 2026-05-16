# Billion-scale Stability & Stress Test Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a high-performance stress test capable of generating 1 Billion nodes and measuring random access latency under memory pressure.

**Architecture:** A standalone Rust executable that uses the core `GraphStore` to fill a massive SHM file incrementally, followed by a randomized probe phase using high-resolution timers.

**Tech Stack:** Rust, `memmap2`, `rand` (SmallRng), `std::time::Instant`.

---

### Task 1: Dependencies & Preparation

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add `rand` to dependencies**

Update `Cargo.toml` to include the `rand` crate for high-speed random sampling.

```toml
[dependencies]
# ... existing ...
rand = { version = "0.8", features = ["small_rng"] }
```

- [ ] **Step 2: Commit**

```bash
git add Cargo.toml
git commit -m "perf: add rand dependency for billion-scale stress testing"
```

---

### Task 2: Implement Billion-Node Generator

**Files:**
- Create: `examples/stress_test_1b.rs`

- [ ] **Step 1: Write the initialization and insertion logic**

Implement the first phase: creating a massive file and filling it with 1B nodes.

```rust
use cntm_graph::GraphStore;
use std::time::Instant;
use rand::{Rng, SeedableRng, rngs::SmallRng};

fn main() -> std::io::Result<()> {
    let path = "stress_test_1b.bin";
    let node_count: usize = 1_000_000_000; // 1 Billion
    let edge_count: usize = 100; // Minimal edges for this test

    println!("🚀 Starting Phase 3 Stress Test: 1 Billion Nodes");
    println!("📂 Target File: {}", path);

    let start = Instant::now();
    let mut store = GraphStore::new(path, node_count, edge_count)?;
    println!("✅ Allocation completed in {:?}", start.elapsed());

    println!("⚡ Filling 1B nodes with data...");
    let fill_start = Instant::now();
    for i in 0..node_count {
        // Simple data pattern
        store.add_node(i as u64, (i % 100) as u16, (i as f32) / 1000.0);

        if i > 0 && i % 100_000_000 == 0 {
            println!("... Progress: {}M nodes ({:?})", i / 1_000_000, fill_start.elapsed());
        }
    }
    println!("✅ Fill completed in {:?}", fill_start.elapsed());
    Ok(())
}
```

- [ ] **Step 2: Commit**

```bash
git add examples/stress_test_1b.rs
git commit -m "perf: implement billion-node generation logic"
```

---

### Task 3: Random Access & Latency Probe

**Files:**
- Modify: `examples/stress_test_1b.rs`

- [ ] **Step 1: Implement the probe phase**

Add code to perform 1 Million random accesses and record latencies.

```rust
// ... inside main after fill ...
    println!("🔍 Starting Random Access Probe (1M samples)...");
    let mut rng = SmallRng::from_entropy();
    let probe_count = 1_000_000;
    let mut latencies = Vec::with_capacity(probe_count);

    for _ in 0..probe_count {
        let idx = rng.gen_range(0..node_count);
        let sample_start = Instant::now();

        // Access Node Data (Hot Path)
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
```

- [ ] **Step 2: Commit**

```bash
git add examples/stress_test_1b.rs
git commit -m "perf: add random access probe and latency reporting"
```

---

### Task 4: Final Validation (Scale-down)

- [ ] **Step 1: Test with 10M nodes first**

Run the test with `node_count = 10_000_000` to ensure no logic errors.

Run: `cargo run --example stress_test_1b` (ensure you change the count or use an env var)
Expected: Program completes, reports sub-microsecond latency, file `stress_test_1b.bin` exists.

- [ ] **Step 2: Cleanup and prepare for 1B run**

Add a final cleanup step to remove the massive test file.

- [ ] **Step 3: Commit final test runner**

```bash
git commit -am "perf: finalized stress test with cleanup and scale-down verification"
```
