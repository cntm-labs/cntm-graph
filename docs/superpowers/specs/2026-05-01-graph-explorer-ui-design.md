# Design Spec: High-Performance Graph Explorer (Phase 3)

## 🎯 Objective
Create a professional, state-of-the-art visual explorer for the `cntm-graph` engine. The UI must handle massive datasets (1M+ visible items) with sub-nanosecond back-end consistency and offer a modern, intuitive UX featuring orthogonal routing (Stepped Lines) and fluid animations.

## 🏗️ Architecture: The High-Performance Island
We utilize a decoupled architecture to separate high-frequency rendering from complex UI state management.

### 1. The Shell (Vue 3 + shadcn-vue + Tailwind)
- **Role:** Management of menus, property panels, search, and application state.
- **Tech Stack:** Vue 3 (Composition API), `shadcn-vue`, `daisyUI`, `Lucide Icons`.
- **UX Pattern:** Skeleton loading for all text-heavy panels to ensure high perceived performance.

### 2. The Core (WebGPU + WGSL)
- **Role:** Ultra-fast rendering of nodes and edges directly from memory.
- **Mechanism:**
    - **Compute Shaders:** For calculating "Stepped Line" paths (orthogonal routing) and particle physics on the GPU.
    - **Vertex/Fragment Shaders:** For "Neon Glow" bloom effects, pulsing animations, and high-fidelity rendering.
    - **Picking Buffer:** For pixel-perfect hover detection at 60+ FPS.

### 3. The Logic Bridge (Rust WASM + Shared Memory)
- **Role:** Orchestrates data flow between the Rust Kernel (`cntm-graph`) and the Browser.
- **Mechanism:** `SharedArrayBuffer` for zero-copy data streaming from the background worker to the WebGPU render loop.

## 🎨 Visual & Interaction Design

### 1. Stepped Lines Routing
- All edges use 90-degree "orthogonal" paths (Blueprint style).
- Prevents visual spaghetti by following a dynamic grid system.

### 2. Dynamic Animation & Highlighting
- **Data Flow:** Animated particles move along lines to represent real-time activity (from `isotime`).
- **Reactive Glow:** Nodes and lines emit a neon bloom effect when hovered or active.
- **Color Grouping:** Automatic color assignment based on `TypeID` (e.g., Logic = Blue, Memory = Purple) for instant cognitive recognition.

### 3. Skeleton States
- Native skeleton components in Vue for sidebar/panels.
- GPU-rendered "shimmering" box placeholders in the graph view during heavy data fetches.

## 🧩 Components to Build
- `GraphCanvas.vue`: The WebGPU entry point.
- `NodeProperties.vue`: shadcn-powered detail panel with skeleton states.
- `core_engine.wasm`: The Rust logic core.
- `stepped_line.wgsl`: The shader for routing logic.

## 🧪 Success Criteria
- [ ] Render 100k nodes and 200k edges at stable 60 FPS.
- [ ] Hover detection works instantly without UI thread lag.
- [ ] Visual grouping (color + layout) significantly reduces "spaghetti" complexity.
