# Unified SIMD Traversal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a high-performance SIMD pipeline for graph traversal, combining type filtering and weight calculation into a single register-level operation.

**Architecture:** Unified SIMD Pipeline using `std::simd` (or `packed_simd`) for 16-wide processing on AVX-512 hardware, with support for masked scoring.

**Tech Stack:** Rust (nightly with `portable_simd`), `std::simd`.

---

### File Mapping
- `src/lib.rs`: Implement the SIMD traversal logic and update data access for tables.
- `benches/traversal_bench.rs`: Create performance benchmarks.

---

### Task 1: Update Data Access & Nightly Setup

**Files:**
- Modify: `src/lib.rs`
- Modify: `rust-toolchain.toml` (create)

- [ ] **Step 1: Set toolchain to nightly**

Create `rust-toolchain.toml` to enable `portable_simd`.

```toml
[toolchain]
channel = "nightly"
components = ["rustfmt", "clippy"]
```

- [ ] **Step 2: Add SIMD imports to src/lib.rs**

```rust
#![feature(portable_simd)]
use std::simd::{u16x16, f32x16, StdFloat, cmp::SimdPartialEq};
```

- [ ] **Step 3: Add slice access to MmapNodeTable**

```rust
impl MmapNodeTable {
    pub fn get_type_slice(&self) -> &[u16] {
        unsafe { std::slice::from_raw_parts(self.type_ids_ptr, self.count) }
    }
    pub fn get_weight_slice(&self) -> &[f32] {
        unsafe { std::slice::from_raw_parts(self.weights_ptr, self.count) }
    }
}
```

- [ ] **Step 4: Commit**

```bash
git add src/lib.rs rust-toolchain.toml
git commit -m "chore: setup nightly rust and SIMD data access"
```

---

### Task 2: Implement Unified SIMD Pipeline

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Implement `find_best_weighted_simd`**

```rust
impl GraphStore {
    pub fn find_best_weighted_simd(&self, target_type: u16) -> (usize, f32) {
        let type_slice = self.nodes.get_type_slice();
        let weight_slice = self.nodes.get_weight_slice();
        
        let mut best_idx = 0;
        let mut best_score = -1.0;

        let target_simd = u16x16::splat(target_type);
        let zero_simd = f32x16::splat(0.0);

        for i in (0..self.nodes.count).step_by(16) {
            if i + 16 > self.nodes.count { break; } // Handle remainder in scalar

            let types = u16x16::from_slice(&type_slice[i..]);
            let weights = f32x16::from_slice(&weight_slice[i..]);
            
            let mask = types.simd_eq(target_simd);
            let scores = mask.select(weights, zero_simd);
            
            let max_score = scores.reduce_max();
            if max_score > best_score {
                // Find index within block (Simplified for prototype)
                for j in 0..16 {
                    if scores[j] == max_score {
                        best_score = max_score;
                        best_idx = i + j;
                    }
                }
            }
        }
        (best_idx, best_score)
    }
}
```

- [ ] **Step 2: Write test for SIMD traversal**

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: implement Unified SIMD Traversal Pipeline"
```

---

### Task 4: Remainder & Scalar Fallback

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Implement scalar fallback for remainder nodes**

- [ ] **Step 2: Commit**

```bash
git add src/lib.rs
git commit -m "feat: add scalar fallback for SIMD traversal"
```

---

### Task 5: Performance Benchmarking

**Files:**
- Create: `benches/traversal_bench.rs`

- [ ] **Step 1: Setup Criterion benchmark**

Compare SIMD vs. Scalar for 1M nodes.

- [ ] **Step 2: Run benchmark**

Run: `cargo bench`

- [ ] **Step 3: Commit**

```bash
git add benches/traversal_bench.rs
git commit -m "bench: add SIMD vs Scalar traversal benchmarks"
```
