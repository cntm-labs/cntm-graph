<div align="center">

# cntm-graph

**Continuum: จุดเชื่อมโยงระหว่างตรรกะเชิงสัญลักษณ์และประสิทธิภาพระดับโครงข่ายประสาท**

[![CI](https://github.com/cntm-labs/cntm-graph/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/cntm-graph/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/cntm-graph/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/cntm-graph/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-active-success)](./)

<!-- Language Badges -->
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Mojo](https://img.shields.io/badge/Mojo-CC0000?style=for-the-badge&logo=mojo&logoColor=white)
![C++](https://img.shields.io/badge/C++-00599C?style=for-the-badge&logo=c%2B%2B&logoColor=white)

<!-- LOD Badges -->
![Rust LOD](https://img.shields.io/badge/Rust%20LOD-0-blue)
![Total LOD](https://img.shields.io/badge/Total%20LOD-0-brightgreen)

</div>

---

[ [English](../README.md) | ภาษาไทย | [日本語](./README.ja.md) | [简体中文](./README.zh.md) ]

Continuum Graph Engine (cntm-graph) คือเอนจินกราฟประสิทธิภาพสูงในระดับ Low-level ที่ออกแบบมาโดยเฉพาะเพื่อทำหน้าที่เป็นเลเยอร์ความจำและการคิดคำนวณสำหรับ AGI พัฒนาด้วยภาษา Rust โดยเน้นการเข้าถึงข้อมูลแบบ Zero-copy เพื่อเชื่อมต่อการใช้เหตุผลเชิงสัญลักษณ์ (Symbolic Reasoning) เข้ากับการประมวลผลแบบ Neural ที่มีความเร็วสูง

## ✨ ฟีเจอร์เด่น (Features)

- 🚀 **Zero-Copy AI-Memory Bridge** — การทำ Memory Mapping (mmap) โดยตรง ช่วยให้ AI Engine (Mojo/C++) เข้าถึงโหนดในกราฟได้ทันทีโดยไม่มีความหน่วง (Zero Latency)
- 🛡️ **Formalized Truth Verification** — ระบบตรวจสอบความถูกต้องผ่าน Lean Proof Assistant เพื่อยืนยันการเปลี่ยนแปลงในกราฟ ป้องกันปัญหา AI Hallucination ตั้งแต่ระดับโครงสร้างข้อมูล
- 📊 **Temporal Evolution Engine** — ทำงานร่วมกับ BlowTime เพื่อบันทึกประวัติการเรียนรู้และความเปลี่ยนแปลงของความรู้ตามลำดับเวลาด้วยการบีบอัดข้อมูลแบบ Delta

## 🛠️ เริ่มต้นใช้งาน (Quick Start)

```bash
# คลอนเรโพซิทอรี
git clone https://github.com/cntm-labs/cntm-graph.git
cd cntm-graph

# บิลด์เอนจิน
cargo build --release

# รันการทดสอบประสิทธิภาพ
cargo bench
```

## 🗺️ การนำทาง (Navigation)

- 🏗️ **[สถาปัตยกรรม (Architecture)](../ARCHITECTURE.md)** — การออกแบบและส่วนประกอบหลัก
- 📅 **[แผนงาน (Roadmap)](../ROADMAP.md)** — ตารางเวลาและเป้าหมายของโปรเจกต์
- 🤝 **[การร่วมพัฒนา (Contributing)](../CONTRIBUTING.md)** — วิธีการเข้าร่วมพัฒนา
- 🌳 **[โครงสร้างโปรเจกต์ (Structure)](../STRUCTURE.tree)** — แผนผังไฟล์ทั้งหมด

## ⚖️ ลิขสิทธิ์ (License)

[MIT](../LICENSE)
