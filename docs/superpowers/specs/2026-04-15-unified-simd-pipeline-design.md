# Unified SIMD Traversal Pipeline Design

- **Date:** 2026-04-15
- **Status:** Draft / Approved Design
- **Topic:** High-Performance Vectorized Traversal (Filtering + Weighting)

## 1. Objective
Achieve sub-nanosecond traversal by processing 16 nodes or edges simultaneously using **AVX-512 SIMD** instructions. The pipeline must filter data by type and calculate importance scores in a single register-level operation.

## 2. Architectural Principles
- **Pipelined Execution:** Minimize data movement between memory and registers. Load once, process multiple times.
- **Masked Operations:** Use SIMD masks to skip branches (if/else), preventing branch misprediction penalties.
- **Hardware Native:** Target `x86_64` AVX-512 for peak performance, with `aarch64` NEON and Scalar fallbacks.

## 3. Technology Stack
- **Rust Features:** `portable_simd` (nightly) or `packed_simd`.
- **Intrinsics:** `_mm512_load_si512`, `_mm512_cmpeq_epu16_mask`, `_mm512_maskz_mov_ps`.
- **Memory Layout:** 64-byte aligned SoA buffers from `GraphStore`.

## 4. The Unified SIMD Pipeline (16-wide)

### Step 1: Parallel Load (64-byte Align)
Load 16 `type_ids` (u16 * 16 = 256 bits) and 16 `weights` (f32 * 16 = 512 bits) from the memory-mapped buffers.
```rust
let types = simd::u16x16::from_slice(&nodes.type_ids[idx..]);
let weights = simd::f32x16::from_slice(&nodes.weights[idx..]);
```

### Step 2: Instant Filtering (Type Mask)
Compare all 16 `type_ids` against the `target_type` in 1 cycle.
```rust
let mask = types.simd_eq(simd::u16x16::splat(target_type));
```

### Step 3: Masked Scoring (Weight Calculation)
Apply the filter mask to the weights. Non-matching nodes get a zero score instantly without branching.
```rust
let scores = mask.select(weights, simd::f32x16::splat(0.0));
```

### Step 4: Horizontal Max/Sum
Quickly find the best node in the 16-node block.
```rust
let best_score = scores.reduce_max();
```

## 5. Implementation Strategy
1. **Feature Gating:** Enable `avx512` target features in Rust.
2. **Buffer Access:** Update `MmapNodeTable` to return `&[T]` slices for safe SIMD loading.
3. **Benchmarking:** Use `criterion` to measure latency vs. traditional scalar loops.

## 6. Success Criteria
- **Latency:** < 0.5ns per node processed within a SIMD block.
- **Throughput:** > 16 Billion nodes/sec filtering capacity.
- **Correctness:** 100% match with scalar implementation results.
