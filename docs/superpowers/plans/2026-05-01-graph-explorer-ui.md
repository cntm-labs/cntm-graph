# High-Performance Graph Explorer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a professional Vue-based web explorer for `cntm-graph` that renders 100k+ nodes using WebGPU with stepped lines and modern animations.

**Architecture:** A decoupled "High-Performance Island" where Vue manages the UI state and a WebGPU canvas handles high-frequency rendering via WGSL shaders.

**Tech Stack:** Vue 3, TailwindCSS, daisyUI, shadcn-vue, WebGPU, Rust (WASM).

---

### Task 1: Scaffold Explorer Shell

**Files:**
- Create: `explorer/` (Project Root)
- Modify: `pixi.toml`

- [ ] **Step 1: Initialize Vue 3 project with Vite**

Run: `npm create vite@latest explorer -- --template vue-ts`
Expected: Folder `explorer/` created with standard Vue/TS structure.

- [ ] **Step 2: Install UI dependencies (Tailwind, daisyUI, shadcn-vue)**

```bash
cd explorer
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
npm install daisyui lucide-vue-next radix-vue
# Install shadcn-vue CLI and init
npx shadcn-vue@latest init -d
```

- [ ] **Step 3: Update Pixi to manage the explorer**

Add a new task to `pixi.toml` for the web dev server.

- [ ] **Step 4: Commit**

```bash
git add explorer/ pixi.toml
git commit -m "feat: scaffold explorer shell with Vue 3, Tailwind, and shadcn"
```

---

### Task 2: WebGPU Foundation

**Files:**
- Create: `explorer/src/core/renderer.ts`
- Create: `explorer/src/components/GraphCanvas.vue`

- [ ] **Step 1: Create WebGPU initialization helper**

Implement basic adapter and device request logic in `renderer.ts`.

- [ ] **Step 2: Implement a basic "Hello Triangle" in the canvas component**

Create `GraphCanvas.vue` that sets up a 100% size `<canvas>` and clears it to the project's background color (`#050505`) using WebGPU.

- [ ] **Step 3: Verify WebGPU Context**

Run the dev server and check if the canvas renders correctly.
Expected: A clean dark canvas with no console errors.

- [ ] **Step 4: Commit**

```bash
git add explorer/src/core/ explorer/src/components/
git commit -m "feat: establish WebGPU rendering context"
```

---

### Task 3: Stepped Lines & Orthogonal Shader

**Files:**
- Create: `explorer/src/shaders/stepped_line.wgsl`

- [ ] **Step 1: Implement the Stepped Line Vertex Shader**

Define a WGSL shader that takes two points (source, target) and calculates a 3-segment orthogonal path (Right-Down-Right).

- [ ] **Step 2: Add Glow/Bloom Fragment Shader**

Implement a pulsing neon effect for lines and nodes.

- [ ] **Step 3: Test rendering 1000 lines**

Pass dummy data from Vue to the shader to verify the routing logic.
Expected: Perfectly aligned 90-degree lines connecting nodes.

- [ ] **Step 4: Commit**

```bash
git add explorer/src/shaders/
git commit -m "feat: implement stepped line shader and neon glow effects"
```

---

### Task 4: Node Table & Skeleton UI

**Files:**
- Create: `explorer/src/components/NodeProperties.vue`

- [ ] **Step 1: Implement Sidebar with Skeleton states**

Use `shadcn-vue` Skeleton components to create a loading state for the node detail panel.

- [ ] **Step 2: Implement Hover Interaction (CPU-side)**

Add mouse move listeners to the canvas to detect node proximity and trigger Vue state updates.

- [ ] **Step 3: Final Integration Demo**

Connect the random data generator to show 10k nodes with animated "data particles" flowing on the lines.

- [ ] **Step 4: Commit**

```bash
git add explorer/src/
git commit -m "feat: add node property panels and finalize explorer prototype"
```
