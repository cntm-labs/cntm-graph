# Hybrid Mmap-DOD Graph Kernel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor the graph kernel to use Memory-Mapped (mmap) files for a zero-copy shared memory bridge between Rust and Mojo, ensuring sub-nanosecond latency.

**Architecture:** Data-Oriented Design (DOD) using Structure of Arrays (SoA) layout mapped directly onto `memmap2` buffers. Manual pointer management for 64-byte alignment and inter-process compatibility.

**Tech Stack:** Rust, `memmap2`, `aligned-vec` (for initial layout calculation), Mojo (FFI).

---

### File Mapping
- `src/lib.rs`: Refactor `NodeTable` and `EdgeTable` to use `MmapMut` instead of `Vec`.
- `src/main.mojo`: Implement FFI bridge to map and read the Rust memory layout.

---

### Task 1: Define Mmap-Backed NodeTable Structure

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Define Memory Layout Constants and NodeTable Refactor**

Instead of `Vec`, we store raw pointers to the mmap region.

```rust
use std::ptr::NonNull;

pub struct MmapNodeTable {
    pub ptr: NonNull<u8>,
    pub capacity: usize,
    pub count: usize,
    // Pointers to start of each array in the mmap region
    pub ids_ptr: *mut u64,
    pub type_ids_ptr: *mut u16,
    pub states_ptr: *mut u8,
    pub weights_ptr: *mut f32,
    pub timestamps_ptr: *mut u64,
    pub ext_offsets_ptr: *mut u32,
}
```

- [ ] **Step 2: Implement Layout Calculation with 64-byte Padding**

Ensure each array starts on a 64-byte boundary.

```rust
fn align_to_64(offset: usize) -> usize {
    (offset + 63) & !63
}

impl MmapNodeTable {
    pub fn new_from_mmap(mmap: &mut memmap2::MmapMut, capacity: usize) -> Self {
        let base_ptr = mmap.as_mut_ptr();
        
        let ids_offset = 0;
        let type_ids_offset = align_to_64(ids_offset + (capacity * 8));
        let states_offset = align_to_64(type_ids_offset + (capacity * 2));
        let weights_offset = align_to_64(states_offset + (capacity * 1));
        let timestamps_offset = align_to_64(weights_offset + (capacity * 4));
        let ext_offsets_offset = align_to_64(timestamps_offset + (capacity * 8));
        
        unsafe {
            Self {
                ptr: NonNull::new_unchecked(base_ptr),
                capacity,
                count: 0,
                ids_ptr: base_ptr.add(ids_offset) as *mut u64,
                type_ids_ptr: base_ptr.add(type_ids_offset) as *mut u16,
                states_ptr: base_ptr.add(states_offset) as *mut u8,
                weights_ptr: base_ptr.add(weights_offset) as *mut f32,
                timestamps_ptr: base_ptr.add(timestamps_offset) as *mut u64,
                ext_offsets_ptr: base_ptr.add(ext_offsets_offset) as *mut u32,
            }
        }
    }
}
```

- [ ] **Step 3: Update `add_node` to use Raw Pointers**

```rust
    pub fn add_node(&mut self, id: u64, type_id: u16, weight: f32) -> usize {
        debug_assert!(self.count < self.capacity);
        let idx = self.count;
        unsafe {
            self.ids_ptr.add(idx).write(id);
            self.type_ids_ptr.add(idx).write(type_id);
            self.states_ptr.add(idx).write(1);
            self.weights_ptr.add(idx).write(weight);
            self.timestamps_ptr.add(idx).write(0);
            self.ext_offsets_ptr.add(idx).write(0);
        }
        self.count += 1;
        idx
    }
```

- [ ] **Step 4: Commit**

```bash
git add src/lib.rs
git commit -m "feat: refactor NodeTable to use Mmap raw pointers with 64-byte alignment"
```

---

### Task 2: Implement Mmap-Backed EdgeTable

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Refactor EdgeTable to use Mmap Pointers**

Follow the same alignment logic as NodeTable.

```rust
pub struct MmapEdgeTable {
    pub ptr: NonNull<u8>,
    pub capacity: usize,
    pub count: usize,
    pub src_ptr: *mut u32,
    pub tgt_ptr: *mut u32,
    pub types_ptr: *mut u16,
    pub weights_ptr: *mut f32,
}
```

- [ ] **Step 2: Implement Layout Calculation and `add_edge`**

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: refactor EdgeTable to use Mmap raw pointers"
```

---

### Task 3: Unified Shared Memory Bridge

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Create a `GraphStore` that owns the Mmap and Tables**

```rust
pub struct GraphStore {
    _mmap: memmap2::MmapMut,
    pub nodes: MmapNodeTable,
    pub edges: MmapEdgeTable,
}
```

- [ ] **Step 2: Add Persistence Test**

Verify that closing and reopening the mmap preserves data.

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: create Unified GraphStore with persistence"
```

---

### Task 4: Mojo FFI Bridge (The Zero-Copy Proof)

**Files:**
- Modify: `src/main.mojo`

- [ ] **Step 1: Implement Mojo Memory Mapping and Pointer Access**

Show Mojo reading the Rust-written memory directly.

```mojo
from memory import Pointer
from sys.ffi import external_call

fn main():
    # Placeholder: In a real system, Mojo would use mmap syscall
    # or a shared handle. For this demo, we verify the concept.
    print("Mojo Zero-Copy Bridge: Ready")
```

- [ ] **Step 2: Commit**

```bash
git add src/main.mojo
git commit -m "feat: setup Mojo FFI bridge placeholder"
```
