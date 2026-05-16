# Continuum Graph User Guide

Welcome to the official user guide for **Continuum Graph**. This document provides everything you need to know to deploy, manage, and scale your AGI memory kernel.

## 1. Environment Setup

### Using Pixi (Recommended for Developers)
We use `pixi` for consistent toolchain management:
```bash
pixi run web    # Starts the Web Explorer
pixi run bench  # Runs performance benchmarks
```

### Using Docker (Production)
```bash
docker-compose up -d
```

## 2. Working with the Graph

### Scaling to Billion Nodes
To verify the system's stability at scale, run the built-in stress test:
```bash
cargo run --release --example stress_test_1b
```
*Note: Ensure you have at least 100GB of free disk space for a full 1B node run.*

### Memory Management & Compaction
Continuum Graph uses an append-only metadata arena. To check for fragmentation and reclaim space:
```bash
cargo run --release --example analyze_memory path/to/your/graph.bin
```

## 3. Visualization (Explorer)
The WebGPU Explorer provides a high-fidelity view of your knowledge graph:
- **Stepped Lines:** Click 'Show Info' to see orthogonal routing that reduces visual clutter.
- **Color Groups:** Nodes are automatically colored based on their `TypeID`.
- **Skeleton States:** Large data fetches will show shimmer placeholders to maintain UI responsiveness.

## 4. Formal Verification
Verify that your local environment satisfies the mathematical safety proofs:
```bash
cd verification
lake build
```

---
For more technical details, refer to the [Architecture Documentation](ARCHITECTURE.md).
