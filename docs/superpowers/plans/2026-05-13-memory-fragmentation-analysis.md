# Memory Fragmentation Analysis & Compaction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement explicit metadata length tracking, a fragmentation analysis tool, and a compaction engine to reclaim wasted memory in the Metadata Arena.

**Architecture:**
1. **Tracking:** Expand the SoA layout to store `length` alongside `offset`.
2. **Analysis:** Sum up all "Alive" lengths and compare against the total arena offset to calculate waste.
3. **Compaction:** Copy active data to a new contiguous block and update offsets atomically.

**Tech Stack:** Rust, Shared Memory, DOD (Structure of Arrays).

---

### Task 1: Infrastructure - Length Tracking

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Update `MmapNodeTable` and `GraphStore`**

Add `ext_lengths_ptr: *mut u32` to `MmapNodeTable`. Update `calculate_mmap_size` and `new_from_ptr` to include this new array. Ensure 64-byte alignment and Guard Segments are maintained.

- [ ] **Step 2: Update `set_node_metadata`**

Modify `set_node_metadata` to record the length of the payload in `ext_lengths_ptr`.

```rust
pub fn set_node_metadata(&mut self, node_idx: usize, payload: &[u8]) -> Result<(), String> {
    // ... current offset logic ...
    unsafe {
        self.nodes.ext_offsets_ptr.add(node_idx).write(offset as u32);
        self.nodes.ext_lengths_ptr.add(node_idx).write(payload.len() as u32); // New line
    }
    // ...
}
```

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: implement explicit length tracking for metadata segments"
```

---

### Task 2: Implementation of Analysis & Reporting

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Implement `analyze_fragmentation`**

Add a method to `GraphStore` that iterates through all nodes and calculates total alive vs dead space.

```rust
pub struct FragmentationReport {
    pub total_bytes: usize,
    pub alive_bytes: usize,
    pub dead_bytes: usize,
    pub ratio: f32,
}

impl GraphStore {
    pub fn analyze_fragmentation(&self) -> FragmentationReport {
        let mut alive_bytes = 0;
        for i in 0..self.nodes.count {
            alive_bytes += unsafe { *self.nodes.ext_lengths_ptr.add(i) } as usize;
        }
        // ... calculation ...
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/lib.rs
git commit -m "feat: add fragmentation analysis logic to GraphStore"
```

---

### Task 3: The Compaction Engine (GC)

**Files:**
- Modify: `src/lib.rs`
- Create: `examples/analyze_memory.rs`

- [ ] **Step 1: Implement `compact_metadata`**

Create a method that re-orders metadata to remove gaps.

```rust
impl GraphStore {
    pub fn compact_metadata(&mut self) -> std::result::Result<(), String> {
        // 1. Create temporary buffer
        // 2. Copy only alive data
        // 3. Update offsets in MmapNodeTable
        // 4. Replace self.metadata.mmap
        Ok(())
    }
}
```

- [ ] **Step 2: Create CLI Tool**

Implement `examples/analyze_memory.rs` to show the fragmentation % and provide a "Compact" button/command.

- [ ] **Step 3: Verification Test**

Write a test that updates one node 10 times, verifies > 90% fragmentation, runs compaction, and verifies < 5% fragmentation.

- [ ] **Step 4: Commit**

```bash
git add .
git commit -m "feat: implement compaction engine and memory analysis utility"
```
