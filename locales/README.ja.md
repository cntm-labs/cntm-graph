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

[ [English](../README.md) | [ภาษาไทย](./README.th.md) | 日本語 | [简体中文](./README.zh.md) ]

> cntm-graphは、AGIのメモリおよび認知レイヤーとして機能するために特別に設計された、高性能な低レベルグラフエンジンです。
High-performance, low-level graph engine for AI memory mapping.

## ✨ 特徴 (Features)
- **Zero-Copy AI-Memory Bridge** — メモリマッピング (mmap) を介して、Mojo/C++エンジンからグラフノードへ遅延なしで直接アクセス。
- **Formalized Truth Verification** — Lean証明アシスタントを統合し、グラフの変更を検証。
- **Temporal Evolution Engine** — デルタ圧縮を用いて知識の時系列的な進化を記録。

## 🛠️ クイックスタート (Quick Start)
```bash
cargo build
cargo test
```

## 🗺️ ナビゲーション (Navigation)
- 🏗️ **[アーキテクチャ (Architecture)](../ARCHITECTURE.md)**
- 📅 **[ロードマップ (Roadmap)](../ROADMAP.md)**
- 🤝 **[貢献する (Contributing)](../CONTRIBUTING.md)**
- 🌳 **[プロジェクト構造 (Structure)](../STRUCTURE.tree)**

## ⚖️ ライセンス (License)
[MIT](../LICENSE)
