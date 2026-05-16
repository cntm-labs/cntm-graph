# Design Spec: Memory Fragmentation Analysis & Compaction (Phase 3)

## 🎯 Objective
Resolve the memory waste issue in the append-only Metadata Arena. By implementing explicit length tracking, analysis tools, and an automated compaction (Garbage Collection) engine, `cntm-graph` will maintain high memory efficiency even after millions of knowledge updates.

## 🏗️ Architecture: The Lifecycle of Metadata
The system follows a three-stage lifecycle for memory management:

### 1. Tracking (Infrastructure)
- **Model:** Upgrade `MmapNodeTable` to store both `offset` and `length`.
- **Change:** Add `ext_lengths_ptr: *mut u32` to the DOD SoA layout.
- **Benefit:** Precise boundaries for every FlatBuffers payload.

### 2. Analysis (Visibility)
- **Logic:** Calculate `Wasted Space = current_offset - Sum(alive_lengths)`.
- **Metrics:**
    - **Fragmentation Ratio:** Percentage of dead space in the arena.
    - **Density Map:** High-level view of which segments are heavily fragmented.

### 3. Compaction (Healing)
- **Mechanism:** "Stop-and-Copy" style compaction.
- **Process:**
    - Create a temporary buffer.
    - Iterate through all nodes, copying "Alive" metadata to the new buffer.
    - Update `ext_offsets` to point to new contiguous locations.
    - Replace the old arena with the compacted one.
- **Trigger:** Configurable threshold (e.g., > 30% fragmentation).

## 🧩 Components

### 1. `src/lib.rs` (Rust Kernel)
- Update `MmapNodeTable` and `GraphStore` to manage length tracking.
- Implement `pub fn analyze_fragmentation(&self) -> FragmentationReport`.
- Implement `pub fn compact_metadata(&mut self) -> Result<(), String>`.

### 2. `examples/analyze_memory.rs` (Tooling)
- A CLI utility to inspect and manually trigger compaction on a live graph file.

## ⚠️ Performance & Safety
- **Alignment:** All compacted data remains 64-byte aligned.
- **Atomic Swap:** Ensure offsets are updated safely to prevent reading partial data during compaction.

## 🧪 Success Criteria
- [ ] `MmapNodeTable` correctly stores and retrieves metadata lengths.
- [ ] Fragmentation report accurately detects dead space after multiple updates to the same node.
- [ ] Compaction reduces arena size back to the sum of alive data (+ alignment overhead).
