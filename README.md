# Continuum Graph

**The High-Performance Memory Kernel for AGI**

[![CI](https://github.com/cntm-labs/cntm-graph/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/cntm-graph/actions/workflows/ci.yml)
[![Performance](https://github.com/cntm-labs/cntm-graph/actions/workflows/performance.yml/badge.svg)](https://github.com/cntm-labs/cntm-graph/actions/workflows/performance.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Continuum Graph is a mission-critical, low-level graph engine designed to serve as the long-term memory layer for AGI systems. It leverages Rust, SIMD acceleration, and Zero-copy shared memory to provide sub-nanosecond traversal speeds.

## ✨ Core Pillars

- 🚀 **Billion-Scale Performance:** Optimized Data-Oriented Design (DOD) supporting 1B+ nodes with massive SIMD throughput.
- 🛡️ **Formal Safety:** Mathematically verified memory boundaries and mutations using the Lean 4 proof assistant.
- 🕰️ **Temporal Awareness:** Real-time delta streaming to **isotime** for full chronological knowledge evolution.
- 🎨 **Visual Intelligence:** A WebGPU-powered explorer with orthogonal (stepped lines) routing for complex reasoning visualization.

## 🚀 Quick Start (Docker)

The fastest way to deploy Continuum Graph is via Docker:

```bash
# Clone and launch
git clone https://github.com/cntm-labs/cntm-graph.git
docker-compose up -d
```

Access the Web Explorer at `http://localhost:5173`.

## 🗺️ Documentation

- 📖 **[User Guide](USER_GUIDE.md)** — Getting started, configuration, and examples.
- 🏗️ **[Architecture](ARCHITECTURE.md)** — Deep dive into the SoA memory layout and FFI bridge.
- 🛡️ **[Security](SECURITY.md)** — Audit protocols and canary guarding details.

---
[ English | [ภาษาไทย](./locales/README.th.md) | [日本語](./locales/README.ja.md) | [简体中文](./locales/README.zh.md) ]
