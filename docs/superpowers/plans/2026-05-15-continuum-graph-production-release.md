# Continuum Graph Production Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Package the entire Continuum Graph ecosystem into a professional, production-ready standard with Docker support and comprehensive documentation.

**Architecture:** A multi-stage Docker build providing a unified runtime for Rust, Mojo, and WebGPU Explorer, backed by "AGI-level" user documentation.

**Tech Stack:** Docker, Rust, Mojo, Vue 3, Pixi.

---

### Task 1: Dockerize Continuum Graph

**Files:**
- Create: `Dockerfile`
- Create: `docker-compose.yml`

- [ ] **Step 1: Write the Multi-stage Dockerfile**

Implement a Dockerfile that handles Rust, Mojo, and Node.js environments.

```dockerfile
# Stage 1: Build Rust & Mojo Core
FROM ubuntu:24.04 as builder
RUN apt-get update && apt-get install -y curl build-essential libssl-dev pkg-config
RUN curl -fsSL https://pixi.sh/install.sh | sh
ENV PATH="/root/.pixi/bin:$PATH"
WORKDIR /app
COPY . .
RUN pixi run cargo build --release

# Stage 2: Build Web Explorer
FROM node:20 as explorer-builder
WORKDIR /app/explorer
COPY explorer/package*.json ./
RUN npm install
COPY explorer/ .
RUN npm run build

# Stage 3: Runtime
FROM ubuntu:24.04
WORKDIR /app
COPY --from=builder /app/target/release/cntm-graph .
COPY --from=explorer-builder /app/explorer/dist ./www
# ... configuration for SHM ...
```

- [ ] **Step 2: Create docker-compose for easy launch**

Configure shared memory limits and GPU pass-through.

- [ ] **Step 3: Commit**

```bash
git add Dockerfile docker-compose.yml
git commit -m "feat: add dockerization for unified continuum graph deployment"
```

---

### Task 2: Official Branding & User Guide

**Files:**
- Modify: `README.md`
- Create: `USER_GUIDE.md`

- [ ] **Step 1: Update README with "Continuum Graph" branding**

Standardize all project naming and add clear value propositions.

- [ ] **Step 2: Write USER_GUIDE.md**

Include sections for:
- Environment Setup (Pixi/Docker)
- Running Benchmarks
- Navigating the Explorer UI
- Interacting with the Zero-copy Bridge

- [ ] **Step 3: Commit**

```bash
git add README.md USER_GUIDE.md
git commit -m "docs: finalize user guide and official continuum graph branding"
```

---

### Task 3: Release Optimization & Final Packaging

**Files:**
- Modify: `Cargo.toml`
- Modify: `pixi.toml`

- [ ] **Step 1: Optimize Cargo Profile**

Enable LTO and strip symbols for production binaries.

```toml
[profile.release]
lto = true
opt-level = 3
codegen-units = 1
panic = 'abort'
strip = true
```

- [ ] **Step 2: Add packaging tasks to Pixi**

Add `docker-up` and `bench-all` tasks.

- [ ] **Step 3: Final Build Verification**

Run `cargo build --release` and `npm run build` one last time.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml pixi.toml
git commit -m "chore: optimize packaging and release configuration"
```
