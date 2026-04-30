# Project Intelligence & Operational Logic

This file is the operational core. Gemini CLI MUST follow these protocols to maintain project integrity.

## 🎯 Architectural Intent & Core Identity
- **Mission:** 'The Autonomous Self-Healing Graph for AGI'.
- **Identity:** A high-performance, low-level graph engine serving as the memory and cognition layer for AGI.
- **Key Technology Stack:** Rust (Core Kernel), Mojo (Cognition Layer/FFI), Shared Memory (SHM/mmap), FlatBuffers (Zero-copy), SIMD (AVX-512/NEON), and Lean Proof Assistant (Formal verification of graph mutations).
- **Core Features:** Sub-nanosecond latency traversal, billion-scale self-healing nodes, and temporal knowledge evolution via Isotime integration.
- **Architectural Philosophy:** Fusing symbolic logic (Lean) with neural performance (Mojo/SIMD) through a zero-copy memory bridge.
- **Current Roadmap Phase:** Q2 2026 - Kernel Development and project bootstrapping.

## 🌐 Ecosystem & Integration
- **Sister Repositories:** @../chronos/ และ @../isotime/
- **Roles in Ecosystem:**
    - **chronos:** เลเยอร์การจัดการ (Orchestration Layer) ที่ใช้ 'cntm-graph' เป็นหน่วยความจำหลัก และ 'isotime' สำหรับการบันทึกข้อมูลเชิงเวลา
    - **Isotime:** เลเยอร์การบันทึกข้อมูลเชิงเวลา (Temporal Persistence Layer) ที่ดึงข้อมูล zero-copy deltas จาก 'cntm-graph' ผ่าน Shared Memory (SHM) แบบเรียลไทม์
- **Operational Protocol:** หากเกิดปัญหาการสร้าง (build-time) หรือข้อขัดแย้งทางสถาปัตยกรรมที่เกิดจากการรวมระบบเหล่านี้ ให้รายงานและติดตามปัญหาเป็น issue ในคลังข้อมูล 'chronos' หรือ 'isotime' ที่เกี่ยวข้องเพื่อการแก้ไขทั้งระบบ
- **Context:** โครงการทั้งสามนี้รวมกันเป็นเฟรมเวิร์ก 'Standard Memory for AGI'

## 🧬 Automated Lifecycle Management
1. **Research Sync:** เมื่อมีการรัน `./scripts/update_notebookLM.sh`:
   - คุณต้องอัปเดต `DESIGN_DECISIONS.md` พร้อม ADR การเพิ่มประสิทธิภาพกราฟใหม่ๆ ที่พบจากการวิจัย
   - **ข้อจำกัด:** รักษาบันทึกของ **10 ADR ล่าสุด**
2. **Logic Verification:** 
   - การเปลี่ยนแปลงโครงสร้างกราฟและการอัปเดตแกนหลักของเอนจินทั้งหมดต้องผ่านการตรวจสอบด้วย Lean ก่อนการคอมมิต
   - ตรวจสอบให้แน่ใจว่าได้กำหนดค่า `LEAN_PATH` อย่างถูกต้องสำหรับสภาพแวดล้อม CI
3. **Performance CI:** 
   - ทุก Pull Request ต้องรัน `Zero-copy latency benchmark` ผ่าน `cargo bench`
   - หากความเร็วในการประมวลผลลดลง (Regression) > 5% ต้องมีการทำเครื่องหมายและตรวจสอบโดย Senior Architect
4. **PR Creation Protocol:** เมื่อได้รับคำสั่งให้สร้าง Pull Request:
   - **สรุป:** วิเคราะห์ข้อความคอมมิตทั้งหมดตั้งแต่การรวม (merge) ครั้งล่าสุดเข้าสู่ `main`
   - **เทมเพลต:** อ่าน `.github/PULL_REQUEST_TEMPLATE.md` และกรอกข้อมูลรายละเอียดให้ครบถ้วน
   - **มอบหมาย:** ตั้งค่าผู้พัฒนาปัจจุบันเป็นผู้รับผิดชอบ (Assignee) โดยอัตโนมัติ

## 🛠️ Tooling & Standards
- **การแปล:** ข้อมูลจำเพาะทางเทคนิคทั้งหมดเป็นภาษาอังกฤษ `locales/` ต้องได้รับการซิงค์เสมอ
- **ความชำนาญในเวิร์กโฟลว์:** ใช้ `/superpower:executing-plans` สำหรับการพัฒนาฟีเจอร์
- **ระบบอัตโนมัติ:** อ้างอิงถึง `.github/workflows/pr_automation.yml` สำหรับการจัดการ PR ฝั่งเซิร์ฟเวอร์
- **Naming Convention:** เรียก 'Isotime' ว่า 'Isotime' เสมอ (Isotime คือ Temporal Persistence Layer ของระบบ)

## 🛡️ Operational Rigor (Lessons from isotime)
- **Priority:** คำสั่งผู้ใช้ (.md files) > กฎ Superpowers > System prompt
- **No Bypassing:** ปฏิบัติตามขั้นตอนทางสถาปัตยกรรมอย่างเคร่งครัด ห้ามข้ามขั้นตอนเพื่อความรวดเร็ว
- **Documentation Parity:** อัปเดตเอกสารทั้งหมด (README, ARCHITECTURE, locales ฯลฯ) ทันทีเมื่อมีการเปลี่ยนแปลงโค้ด
- **Zero Technical Debt:** ห้ามใช้ `#[allow(dead_code)]` หรือโค้ดที่ไม่ได้ใช้งาน ให้แก้ไขที่ต้นเหตุแทนการปิดคำเตือน
- **Pre-Commit Safeguards:** อัปเดต `STRUCTURE.tree` และรันการจัดรูปแบบ (`cargo fmt`/`mojo format`) ก่อนการ Commit
- **1% Skill Rule:** เปิดใช้งาน skill ที่เกี่ยวข้องแม้ว่าจะมีโอกาสเพียง 1% ที่จะนำไปใช้ได้
- **Git Health:** รักษา `.gitignore` ให้แข็งแกร่งเพื่อป้องกันการ Commit ไฟล์ขยะ (ไฟล์ระบบ, IDE, local test data)

## 📂 Template Inventory & Key References
- **Managed Templates:** ARCHITECTURE.md, ROADMAP.md, CONTRIBUTING.md, DESIGN_DECISIONS.md, STRUCTURE.tree, SECURITY.md, LICENSE.md, FAQ.md, GOVERNANCE.md, SUPPORT.md, TROUBLESHOOTING.md, PHILOSOPHY.md, MANIFESTO.md, และ `locales/README.{th,ja,zh}.md`
- **Key Context Files:** ARCHITECTURE.md, PHILOSOPHY.md, MANIFESTO.md, และ VISION.md.
