# Hybrid DOD Graph Kernel Design

- **Date:** 2026-04-15
- **Status:** Draft / Approved Design
- **Topic:** High-Performance Graph Kernel for AGI Memory

## 1. Objective
Build a sub-nanosecond latency graph engine capable of handling billions of nodes and edges. The kernel must serve as a zero-copy memory bridge between **Rust (Core Engine)** and **Mojo (AI Cognition Layer)**.

## 2. Architectural Principles
- **Data-Oriented Design (DOD):** Prioritize data layout over object-oriented abstractions to maximize CPU cache efficiency.
- **Structure of Arrays (SoA):** Separate node/edge attributes into contiguous memory buffers to minimize cache misses during traversal.
- **SIMD-First:** Align memory to 64-byte boundaries to fully utilize AVX-512 (processing 16 nodes per instruction).
- **Hybrid Extensibility:** Maintain a "Hot Path" for core traversal fields (Fixed-size) and a "Cold Path" for flexible metadata (FlatBuffers).

## 3. Technology Stack
- **Languages:** Rust (Kernel), Mojo (FFI/Cognition), Lean (Formal Verification).
- **Memory:** Shared Memory (SHM) via `mmap`.
- **Serialization:** FlatBuffers (Zero-copy).
- **Acceleration:** SIMD (AVX-512/NEON).

## 4. Core Data Structures

### 4.1 Node Table (SoA Layout)
Instead of a single `struct Node`, properties are split across parallel arrays for optimal pre-fetching:

| Field | Type | Alignment | Purpose |
| :--- | :--- | :--- | :--- |
| `ids` | `u64` | 8-byte | Global Unique Identifier |
| `type_ids` | `u16` | 2-byte | Schema-defined node type |
| `states` | `u8` | 1-byte | Lifecycle: Active, Deleted, Locked |
| `weights` | `f32` | 4-byte | Neural importance (Hot Path) |
| `timestamps` | `u64` | 8-byte | BlowTime temporal sync |
| `ext_offsets` | `u32` | 4-byte | Offset to FlatBuffers metadata (Cold Path) |

### 4.2 Edge Table
Edges follow a similar SoA pattern to enable fast neighborhood scans:

- `source_indices`: `u32`
- `target_indices`: `u32`
- `edge_types`: `u16`
- `edge_weights`: `f32`

### 4.3 SIMD Alignment
Structures involved in mathematical operations will use explicit alignment:

```rust
#[repr(C, align(64))]
pub struct AlignedWeightBlock {
    pub values: [f32; 16],
}
```

## 5. Memory & FFI Strategy
- **Shared Memory Bridge:** The `NodeTable` and `EdgeTable` will reside in a shared memory segment mapped by both Rust and Mojo.
- **Zero-Copy Access:** Mojo will access the raw pointers directly using `#[repr(C)]` compatibility, bypassing any serialization overhead.
- **Mutation Verification:** All graph mutations (Add/Delete/Update) must be verifiable via **Lean** to ensure structural integrity in self-healing scenarios.

## 6. Implementation Plan Highlights
1. **Scaffold Rust Kernel:** Implement `src/lib.rs` with the SoA structures.
2. **SHM Integration:** Setup `mmap` for the core tables.
3. **Mojo Bridge:** Create the Mojo FFI definitions to read the Rust memory layout.
4. **SIMD Benchmarking:** Verify sub-nanosecond traversal performance.

## 7. Success Criteria
- **Latency:** < 1ns per node traversal step.
- **Scalability:** Handle > 1 Billion nodes without performance degradation.
- **Integrity:** 100% formal verification pass rate for core mutations.
