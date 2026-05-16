# FFI Boundary Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a multi-layered security system using Canary Tokens and Formal Proofs to protect the Shared Memory bridge between Rust, Mojo, and C.

**Architecture:**
1. **Canary Guarding:** 64-byte blocks with fixed patterns are placed between every SoA array in SHM.
2. **Formal Verification:** Lean 4 proofs ensure that pointer arithmetic never reaches these guard segments.

**Tech Stack:** Rust, C, Lean 4, Shared Memory.

---

### Task 1: Canary Implementation in Rust Kernel

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Update offset calculation to include Guards**

Modify `align_to_64` and add a constant for `GUARD_SIZE = 64`. Update `calculate_mmap_size` to insert a guard after every segment.

```rust
const CANARY_PATTERN: u64 = 0xDEADBEEFCAFEBABE;
const GUARD_SIZE: usize = 64;

// In MmapNodeTable::new_from_ptr and calculate_mmap_size
// Insert guards between ids, types, states, weights...
```

- [ ] **Step 2: Implement `initialize_guards` and `verify_canaries`**

```rust
impl GraphStore {
    pub fn verify_canaries(&self) -> bool {
        // Logic to check all guard segments in SHM
        true
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: add canary guard segments to SHM layout"
```

---

### Task 2: Fast Canary Check in C Helper

**Files:**
- Modify: `src/helper.c`

- [ ] **Step 1: Add `verify_canary` function to C helper**

Implement a high-performance check that can be called by Mojo or the loader.

```c
int verify_canary(void* ptr, size_t offset) {
    unsigned long long* canary = (unsigned long long*)((char*)ptr + offset);
    return *canary == 0xDEADBEEFCAFEBABE;
}
```

- [ ] **Step 2: Rebuild helper library**

Run: `gcc -shared -o src/libhelper.so -fPIC src/helper.c`

- [ ] **Step 3: Commit**

```bash
git add src/helper.c
git commit -m "feat: implement C-side canary verification"
```

---

### Task 3: Formal Proof of Guard Isolation

**Files:**
- Modify: `verification/Memory.lean`
- Modify: `verification/Safety.lean`

- [ ] **Step 1: Update Segment model in Lean**

Add a `is_guard : Bool` field to the `Segment` structure.

- [ ] **Step 2: Prove Non-overlap with Guards**

Theorem: Proving that any data access at index `i` is strictly less than the start of the next `Guard` segment.

- [ ] **Step 3: Commit**

```bash
git add verification/
git commit -m "formal: prove memory isolation and guard-safe indexing"
```

---

### Task 4: Integration Test (Canary Breach)

**Files:**
- Create: `examples/verify_guards.rs`

- [ ] **Step 1: Implement a test that purposely attempts to overflow**

Write a test that writes past a segment and verifies that `verify_canaries()` returns `false`.

- [ ] **Step 2: Run verification**

Run: `cargo run --example verify_guards`
Expected: Output showing "Canary Breach Detected!" when data overflows.

- [ ] **Step 3: Commit**

```bash
git add examples/verify_guards.rs
git commit -m "test: add integration test for canary breach detection"
```
