<script setup lang="ts">
import GraphCanvas from './components/GraphCanvas.vue'
import { ref } from 'vue'

const isSidebarOpen = ref(true)
</script>

<template>
  <div class="flex h-screen w-screen bg-[#050505] text-white font-sans overflow-hidden">
    <!-- Main High-Performance Island -->
    <main class="flex-1 relative h-full">
      <GraphCanvas />

      <!-- Top Navigation / Overlay -->
      <nav class="absolute top-4 left-4 flex gap-4 pointer-events-none">
        <div class="pointer-events-auto bg-black/60 border border-zinc-800 p-2 rounded-lg backdrop-blur-md">
          <h1 class="text-xs font-bold tracking-tighter text-zinc-400">CNTM-GRAPH EXPLORER <span class="text-[8px] font-normal opacity-50">V0.1.0-DEV</span></h1>
        </div>
      </nav>
    </main>

    <!-- Side UI Panel (Vue + Tailwind) -->
    <aside
      v-if="isSidebarOpen"
      class="w-80 bg-[#0a0a0a] border-l border-zinc-800 flex flex-col h-full shadow-2xl transition-all duration-300"
    >
      <div class="p-6 border-b border-zinc-800">
        <h2 class="text-sm font-semibold tracking-wide text-zinc-100 uppercase">Node Properties</h2>
        <p class="text-[10px] text-zinc-500 mt-1">Real-time SHM Metadata</p>
      </div>

      <!-- Skeleton Loading State Simulation -->
      <div class="flex-1 p-6 space-y-6">
        <div class="space-y-3">
          <div class="h-2 w-1/3 bg-zinc-800 rounded animate-pulse"></div>
          <div class="h-8 w-full bg-zinc-900 border border-zinc-800 rounded-md animate-pulse"></div>
        </div>

        <div class="space-y-4">
          <div class="h-2 w-1/2 bg-zinc-800 rounded animate-pulse"></div>
          <div class="grid grid-cols-2 gap-4">
            <div class="h-20 bg-zinc-900 border border-zinc-800 rounded-lg animate-pulse"></div>
            <div class="h-20 bg-zinc-900 border border-zinc-800 rounded-lg animate-pulse"></div>
          </div>
        </div>

        <div class="pt-10">
          <div class="flex items-center gap-2 mb-4">
            <div class="w-2 h-2 rounded-full bg-emerald-500"></div>
            <span class="text-[10px] font-mono text-zinc-400 uppercase">System Status</span>
          </div>
          <div class="space-y-2">
            <div class="flex justify-between text-[10px] font-mono text-zinc-500">
              <span>MEMORY MAPPED:</span>
              <span class="text-zinc-300">ACTIVE</span>
            </div>
            <div class="flex justify-between text-[10px] font-mono text-zinc-500">
              <span>ZERO-COPY BRIDGE:</span>
              <span class="text-zinc-300">READY</span>
            </div>
          </div>
        </div>
      </div>

      <div class="p-4 border-t border-zinc-800 bg-black/40">
        <button class="w-full py-2 bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-bold rounded-md transition-colors shadow-lg shadow-emerald-900/20">
          ANALYZE GRAPH
        </button>
      </div>
    </aside>
  </div>
</template>

<style>
body, html {
  margin: 0;
  padding: 0;
  background: #000;
  overflow: hidden;
}

::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: #222;
  border-radius: 10px;
}
</style>
