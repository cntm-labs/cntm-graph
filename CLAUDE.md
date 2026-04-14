# Project Intelligence & Operational Logic

This file is the operational core. Claude Code MUST follow these protocols to maintain project integrity.

## 🎯 Architectural Intent
- **Core Mission:** The Autonomous Self-Healing Graph for AGI
- **Primary Stack:** Rust, Shared Memory (SHM), FlatBuffers, SIMD (AVX-512), Mojo FFI
- **System Nature:** High-performance, low-level graph engine for AI memory mapping.

## 🧬 Automated Lifecycle Management
1. **Research Sync:** When `./scripts/update_notebookLM.sh` is executed:
   - You MUST update `DESIGN_DECISIONS.md` with new graph optimization ADRs found in research.
   - **Constraint:** Maintain a rolling log of the **latest 10 ADRs**.
2. **Logic Verification:** 
   - All new graph mutations and engine kernel updates MUST pass Lean verification before commit.
   - Ensure `LEAN_PATH` is configured correctly for the CI environment.
3. **Performance CI:** 
   - Every Pull Request MUST execute the `Zero-copy latency benchmark` via `cargo bench`.
   - Regressions > 5% in traversal speed must be flagged and reviewed by a Senior Architect.
4. **PR Creation Protocol:** When instructed to create a Pull Request:
   - **Summarize:** Analyze all commit messages since the last merge to `main`.
   - **Template:** Read `.github/PULL_REQUEST_TEMPLATE.md` and populate it with a detailed description.
   - **Assign:** Automatically set the current developer as the Assignee.

## 🛠️ Tooling & Standards
- **Translation:** All technical specifications are English. `locales/` MUST be kept in sync.
- **Workflow Mastery:** Use `/superpower:executing-plans` for feature work.
- **Automation:** Refer to `.github/workflows/pr_automation.yml` for server-side PR handling.

## 📂 Template Inventory
You manage: ARCHITECTURE.md, ROADMAP.md, CONTRIBUTING.md, DESIGN_DECISIONS.md, STRUCTURE.tree, SECURITY.md, LICENSE.md, FAQ.md, GOVERNANCE.md, SUPPORT.md, TROUBLESHOOTING.md, PHILOSOPHY.md, MANIFESTO.md, and `locales/README.{th,ja,zh}.md`.
