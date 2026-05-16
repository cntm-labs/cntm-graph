# Design Spec: FFI Boundary Hardening (Phase 3 Audit)

## 🎯 Objective
Strengthen the security and integrity of the data bridge between Rust, Mojo, and C. The primary goal is to prevent Memory Corruption, Buffer Overflows, and Pointer Collisions by implementing a multi-layered defense system using Runtime Canary Tokens and Formal Proofs.

## 🏗️ Architecture
The defense strategy is split into Physical (Runtime) and Logical (Compile-time) layers.

### 1. Physical Layer: Canary Token Guarding
- **Mechanism:** Insert 64-byte "Guard Segments" between every data array (SoA components) in the shared memory file.
- **Canary Pattern:** A fixed, verifiable bit-pattern (e.g., `0xDEADBEEFCAFEBABE`) is written into these guard segments.
- **Verification:**
    - Rust Kernel checks canaries after every batch write.
    - C Helper/Mojo checks canaries before accessing high-frequency nodes.
- **Response:** Any violation triggers an immediate `SAFE_SHUTDOWN` and logs a security alert.

### 2. Logical Layer: Lean Pointer Specification
- **Formalization:** Extend the Lean 4 model to include "Pointer Permissions".
- **Proofs:**
    - **In-Bounds Invariant:** Prove that the address returned for any node index $i$ is strictly within $[S, S+L)$ where $L$ is the length of the data segment excluding guards.
    - **Isolation Theorem:** Prove that writing to Segment A cannot affect the bits in Segment B or the Guard Segments.

## 🧩 Components

### 1. `src/lib.rs` (Rust)
- Update `calculate_mmap_size` to account for Guard Segments.
- Implement `initialize_guards()` to populate canary tokens.
- Implement `validate_shm_integrity()` to scan for corruption.

### 2. `src/helper.c` (C Helper)
- Implement `fast_canary_check()` using SIMD to quickly verify guards before memory mapping.

### 3. `verification/Safety.lean` (Lean 4)
- Add axioms for `SegmentGuard` and prove that the base pointer + offset calculation is "Guard-Safe".

## ⚠️ Performance Impact
- **Memory:** Adds ~64 bytes per segment (Negligible for 1B nodes).
- **Latency:** Canary checks are performed asynchronously or before/after batch operations, keeping the hot-path traversal at sub-nanosecond speeds.

## 🧪 Success Criteria
- [ ] 100% of buffer overflows between segments detected in stress tests.
- [ ] Formal proof of "Guard-Safe" pointer arithmetic completed in Lean.
- [ ] No performance regression in core traversal speed.
