<div align="center">

# cntm-graph

**High-performance, low-level graph engine for AI memory mapping.**

[![CI](https://github.com/cntm-labs/cntm-graph/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/cntm-graph/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/cntm-graph/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/cntm-graph/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-active-success)](./)

<!-- Language Badges -->
![Rust](https://img.shields.io/badge/language-Rust-orange.svg) ![Mojo](https://img.shields.io/badge/language-Mojo-red.svg)

<!-- LOD Badges -->
![Rust LOD](https://img.shields.io/badge/Rust_LOD-0-blue.svg) ![Total LOD](https://img.shields.io/badge/Total_LOD-0-brightgreen.svg)

</div>

---

[ [English](../README.md) | [ภาษาไทย](./README.th.md) | [日本語](./README.ja.md) | 简体中文 ]

> cntm-graph 是一款高性能、低级图引擎，专为 AGI 的内存和认知层设计。它采用 Rust 开发，专注于零拷贝数据遍历。
High-performance, low-level graph engine for AI memory mapping.

## ✨ 特性 (Features)
- **Zero-Copy AI-Memory Bridge** — 通过内存映射 (mmap) 实现从 Mojo/C++ 引擎对图节点的零延迟直接访问。
- **Formalized Truth Verification** — 集成 Lean 证明辅助工具验证图变更。
- **Temporal Evolution Engine** — 利用增量压缩捕捉知识随时间推移的演变过程。

## 🛠️ 快速开始 (Quick Start)
```bash
cargo build
cargo test
```

## 🗺️ 导航 (Navigation)
- 🏗️ **[架构 (Architecture)](../ARCHITECTURE.md)**
- 📅 **[路线图 (Roadmap)](../ROADMAP.md)**
- 🤝 **[贡献 (Contributing)](../CONTRIBUTING.md)**
- 🌳 **[项目结构 (Structure)](../STRUCTURE.tree)**

## ⚖️ 许可证 (License)
[MIT](../LICENSE)
