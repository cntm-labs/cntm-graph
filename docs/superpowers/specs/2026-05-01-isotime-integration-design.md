# Design Spec: isotime Integration (Phase 2)

## 🎯 Objective
Establish a high-performance, zero-copy communication bridge between `cntm-graph` (Memory Kernel) and `isotime` (Temporal Persistence Layer). This allows the system to record a precise history of graph evolutions with sub-microsecond synchronization overhead.

## 🏗️ Architecture: Hybrid Delta Bridge
To balance reliability and throughput, the integration uses a dual-channel synchronization model in Shared Memory (SHM).

### 1. Structural Channel (Push/Event-driven)
- **Purpose:** Captures critical changes (Node/Edge Additions, Deletions, Schema updates).
- **Mechanism:** A dedicated **Circular Ring Buffer** (`MmapDeltaLog`) in SHM.
- **Data Flow:** `cntm-graph` writes a compact `EventPacket` -> Increments Atomic Tail -> `isotime` reads from Atomic Head.

### 2. Data Channel (Pull/Scanning)
- **Purpose:** Captures high-frequency updates (Weight changes, Dynamic states).
- **Mechanism:** A **Dirty Bitmask** integrated into the core `MmapNodeTable`.
- **Data Flow:** `cntm-graph` sets bit on update -> `isotime` periodically scans bitmask via SIMD -> Extracts values -> Clears bits.

## 🧩 Components

### 1. `MmapDeltaLog` (Rust)
- A lock-free ring buffer structure managing a pre-allocated segment of the memory-mapped file.
- Support for `EventPacket` types: `NODE_ADD`, `NODE_DEL`, `EDGE_ADD`, `EDGE_DEL`, `META_UPDATE`.

### 2. `DOD Extension: DirtyMask` (Rust/SIMD)
- Adds an array of `u64` (acting as bitsets) to the Node Table.
- Aligned to 64 bytes for rapid scanning by `isotime`'s cognition layer.

### 3. `Isotime Handshake` (FFI/Mojo)
- Provides pointers and offsets to the external `isotime` process to allow direct reading of the SHM segments.

## ⚠️ Safety & Consistency
- **Atomic Pointers:** All indices for the ring buffer use `AtomicUsize` to prevent race conditions.
- **Alignment:** Both the log and the mask follow the project's 64-byte alignment mandate.

## 🧪 Success Criteria
- [ ] `MmapDeltaLog` successfully records and retrieves 1M events without corruption.
- [ ] SIMD scan of 1B dirty bits completes in < 5ms.
- [ ] `isotime` can reconstruct the graph state from the recorded deltas.
