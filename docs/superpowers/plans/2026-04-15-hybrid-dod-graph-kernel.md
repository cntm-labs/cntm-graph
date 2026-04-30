# Hybrid DOD Graph Kernel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a high-performance, data-oriented graph kernel in Rust with SoA layout and SIMD alignment, serving as a zero-copy memory bridge for Mojo.

**Architecture:** Data-Oriented Design (DOD) using Structure of Arrays (SoA) for cache locality, 64-byte alignment for AVX-512, and a hybrid Hot/Cold path for metadata.

**Tech Stack:** Rust, `memmap2`, `flatbuffers`, SIMD (AVX-512/NEON).

---

### File Mapping
- `Cargo.toml`: Project dependencies (memmap2, flatbuffers).
- `src/lib.rs`: Main kernel implementation (NodeTable, EdgeTable, SIMD blocks).
- `tests/kernel_test.rs`: Integration tests for performance and alignment.

---

### Task 1: Project Setup & Dependencies

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add dependencies to Cargo.toml**

```toml
[dependencies]
memmap2 = "0.9"
flatbuffers = "23.5.26"
aligned-vec = "0.5" # For SIMD-aligned memory management
```

- [ ] **Step 2: Run cargo check to verify**

Run: `cargo check`
Expected: Success

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add dependencies for graph kernel"
```

---

### Task 2: Define Aligned SIMD Blocks

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing test for alignment**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::align_of;

    #[test]
    fn test_weight_block_alignment() {
        assert_eq!(align_of::<AlignedWeightBlock>(), 64);
    }
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test test_weight_block_alignment`
Expected: FAIL (AlignedWeightBlock not defined)

- [ ] **Step 3: Implement AlignedWeightBlock**

```rust
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct AlignedWeightBlock {
    pub values: [f32; 16],
}

impl AlignedWeightBlock {
    pub fn new() -> Self {
        Self { values: [0.0; 16] }
    }
}
```

- [ ] **Step 4: Run test to verify pass**

Run: `cargo test test_weight_block_alignment`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "feat: add SIMD-aligned weight blocks"
```

---

### Task 3: Implement NodeTable (SoA Layout)

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing test for NodeTable**

```rust
#[test]
fn test_node_table_addition() {
    let mut table = NodeTable::new(1024);
    let idx = table.add_node(12345, 1, 0.85);
    assert_eq!(table.ids[idx], 12345);
    assert_eq!(table.weights[idx], 0.85);
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test test_node_table_addition`
Expected: FAIL (NodeTable not defined)

- [ ] **Step 3: Implement NodeTable Struct and add_node**

```rust
pub struct NodeTable {
    pub ids: Vec<u64>,
    pub type_ids: Vec<u16>,
    pub states: Vec<u8>,
    pub weights: Vec<f32>,
    pub timestamps: Vec<u64>,
    pub ext_offsets: Vec<u32>,
    pub capacity: usize,
    pub count: usize,
}

impl NodeTable {
    pub fn new(capacity: usize) -> Self {
        Self {
            ids: Vec::with_capacity(capacity),
            type_ids: Vec::with_capacity(capacity),
            states: Vec::with_capacity(capacity),
            weights: Vec::with_capacity(capacity),
            timestamps: Vec::with_capacity(capacity),
            ext_offsets: Vec::with_capacity(capacity),
            capacity,
            count: 0,
        }
    }

    pub fn add_node(&mut self, id: u64, type_id: u16, weight: f32) -> usize {
        let idx = self.count;
        self.ids.push(id);
        self.type_ids.push(type_id);
        self.states.push(1); // Active
        self.weights.push(weight);
        self.timestamps.push(0); // Placeholder for Isotime
        self.ext_offsets.push(0);
        self.count += 1;
        idx
    }
}
```

- [ ] **Step 4: Run test to verify pass**

Run: `cargo test test_node_table_addition`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "feat: implement NodeTable with SoA layout"
```

---

### Task 4: Implement EdgeTable (SoA Layout)

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing test for EdgeTable**

```rust
#[test]
fn test_edge_table_addition() {
    let mut table = EdgeTable::new(1024);
    let idx = table.add_edge(0, 1, 5, 0.5);
    assert_eq!(table.source_indices[idx], 0);
    assert_eq!(table.target_indices[idx], 1);
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test test_edge_table_addition`
Expected: FAIL (EdgeTable not defined)

- [ ] **Step 3: Implement EdgeTable Struct**

```rust
pub struct EdgeTable {
    pub source_indices: Vec<u32>,
    pub target_indices: Vec<u32>,
    pub edge_types: Vec<u16>,
    pub edge_weights: Vec<f32>,
    pub count: usize,
}

impl EdgeTable {
    pub fn new(capacity: usize) -> Self {
        Self {
            source_indices: Vec::with_capacity(capacity),
            target_indices: Vec::with_capacity(capacity),
            edge_types: Vec::with_capacity(capacity),
            edge_weights: Vec::with_capacity(capacity),
            count: 0,
        }
    }

    pub fn add_edge(&mut self, src: u32, tgt: u32, edge_type: u16, weight: f32) -> usize {
        let idx = self.count;
        self.source_indices.push(src);
        self.target_indices.push(tgt);
        self.edge_types.push(edge_type);
        self.edge_weights.push(weight);
        self.count += 1;
        idx
    }
}
```

- [ ] **Step 4: Run test to verify pass**

Run: `cargo test test_edge_table_addition`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "feat: implement EdgeTable with SoA layout"
```

---

### Task 5: Memory Mapping (mmap) Scaffold

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing test for Mmap**

```rust
#[test]
fn test_mmap_initialization() {
    let result = init_shared_memory("test_graph.bin", 1024 * 1024);
    assert!(result.is_ok());
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test test_mmap_initialization`
Expected: FAIL (init_shared_memory not defined)

- [ ] **Step 3: Implement init_shared_memory**

```rust
use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::io::{Result, Write};

pub fn init_shared_memory(path: &str, size: usize) -> Result<MmapMut> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;
    file.set_len(size as u64)?;
    unsafe { MmapMut::map_mut(&file) }
}
```

- [ ] **Step 4: Run test to verify pass**

Run: `cargo test test_mmap_initialization`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "feat: add shared memory mmap initialization"
```
