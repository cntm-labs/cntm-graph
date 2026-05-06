# isotime Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a zero-copy delta bridge between `cntm-graph` and `isotime` using a hybrid Push/Pull model in Shared Memory.

**Architecture:**
1. **Push:** A Circular Ring Buffer (`MmapDeltaLog`) captures structural changes (Add/Del).
2. **Pull:** A SIMD-ready Dirty Bitmask in the Node Table tracks weight updates for rapid scanning.

**Tech Stack:** Rust, Shared Memory (mmap), Atomics, SIMD.

---

### Task 1: Structural Channel - Circular Ring Buffer

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Define Event types and Delta Log structure**

Add `EventPacket` and `MmapDeltaLog` to `src/lib.rs`.

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum GraphEvent {
    NodeAdded { id: u64, type_id: u16 },
    EdgeAdded { src: u32, tgt: u32 },
}

#[repr(C)]
pub struct EventPacket {
    pub timestamp: u64,
    pub event: GraphEvent,
}

pub struct MmapDeltaLog {
    pub mmap: MmapMut,
    pub capacity: usize,
    // Header (first 16 bytes): Head (8b), Tail (8b)
}

impl MmapDeltaLog {
    pub fn new(mmap: MmapMut, capacity: usize) -> Self {
        Self { mmap, capacity }
    }

    pub fn push(&mut self, event: GraphEvent) {
        // SAFETY: Simple pointer math for append-only log in this phase
        // Logic for atomic tail increment and wrap-around...
    }
}
```

- [ ] **Step 2: Initialize Delta Log in GraphStore**

Update `GraphStore::new` to create a `delta_log.bin` file.

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: add basic MmapDeltaLog and GraphEvent definitions"
```

---

### Task 2: Data Channel - Dirty Bitmask

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Expand MmapNodeTable with Dirty Bits**

Add `dirty_mask_ptr: *mut u64` to `MmapNodeTable`. Each bit represents one node.

```rust
// In MmapNodeTable::new_from_ptr
let mask_offset = align_to_64(ext_offsets_offset + (capacity * 4));
// ...
dirty_mask_ptr: base_ptr.add(mask_offset) as *mut u64,
```

- [ ] **Step 2: Update add_node and set_node_metadata to set bits**

```rust
pub fn mark_dirty(&mut self, idx: usize) {
    let word_idx = idx / 64;
    let bit_idx = idx % 64;
    unsafe {
        let val = self.dirty_mask_ptr.add(word_idx).read();
        self.dirty_mask_ptr.add(word_idx).write(val | (1 << bit_idx));
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: implement Dirty Bitmask for tracking high-frequency updates"
```

---

### Task 3: Integration & Mock Watcher

**Files:**
- Modify: `src/lib.rs`
- Create: `examples/isotime_mock_watcher.rs`

- [ ] **Step 1: Auto-push events on GraphStore mutations**

Update `store.add_node` to automatically call `self.delta_log.push(...)`.

- [ ] **Step 2: Create Mock Watcher**

Write an example that maps the same `delta_log.bin` and prints incoming events.

- [ ] **Step 3: Verification Test**

Create a test `test_isotime_handshake` that writes 5 nodes and verifies the Delta Log contains 5 events.

- [ ] **Step 4: Commit**

```bash
git add .
git commit -m "feat: integrate delta logging and add isotime mock watcher"
```
