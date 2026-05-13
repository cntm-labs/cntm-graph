<script setup lang="ts">
import GraphCanvas from './components/GraphCanvas.vue'
import NodeProperties from './components/NodeProperties.vue'
import { ref, onMounted } from 'vue'

const isSidebarOpen = ref(true)
const isLoadingNode = ref(true)
const selectedNodeId = ref('AGI_NODE_007')

onMounted(() => {
  // Simulate loading delay for the skeleton UI demo
  setTimeout(() => {
    isLoadingNode.value = false
  }, 2500)
})

const toggleSidebar = () => {
  isSidebarOpen.value = !isSidebarOpen.value
}
</script>

<template>
  <div class="flex h-screen w-screen bg-[#050505] text-white font-sans overflow-hidden">
    <!-- Main High-Performance Island -->
    <main class="flex-1 relative h-full">
      <GraphCanvas />

      <!-- Top Navigation / Overlay -->
      <nav class="absolute top-4 left-4 flex gap-4 pointer-events-none">
        <div class="pointer-events-auto bg-black/60 border border-zinc-800 p-2 rounded-lg backdrop-blur-md flex items-center gap-4">
          <h1 class="text-xs font-bold tracking-tighter text-zinc-100">CNTM-GRAPH <span class="text-emerald-500">EXPLORER</span></h1>
          <div class="h-4 w-[1px] bg-zinc-800"></div>
          <button @click="toggleSidebar" class="text-[10px] text-zinc-400 hover:text-white transition-colors uppercase font-mono">
            {{ isSidebarOpen ? 'Hide Info' : 'Show Info' }}
          </button>
        </div>
      </nav>

      <!-- Bottom Status Bar -->
      <div class="absolute bottom-4 left-4 pointer-events-none">
        <div class="bg-black/40 px-3 py-1.5 rounded border border-zinc-800/50 backdrop-blur-sm">
          <p class="text-[9px] font-mono text-zinc-500 uppercase tracking-widest">
            Nodes: <span class="text-zinc-300">1,000,000</span> | Edges: <span class="text-zinc-300">5,000,000</span> | Latency: <span class="text-emerald-400">0.45ns</span>
          </p>
        </div>
      </div>
    </main>

    <!-- Side UI Panel (Vue + Tailwind) -->
    <transition
      enter-active-class="transition-transform duration-500 ease-out"
      enter-from-class="translate-x-full"
      enter-to-class="translate-x-0"
      leave-active-class="transition-transform duration-300 ease-in"
      leave-from-class="translate-x-0"
      leave-to-class="translate-x-full"
    >
      <aside
        v-if="isSidebarOpen"
        class="w-80 bg-[#0a0a0a] border-l border-zinc-800 flex flex-col h-full shadow-2xl z-20"
      >
        <div class="p-6 border-b border-zinc-800 flex justify-between items-start">
          <div>
            <h2 class="text-sm font-semibold tracking-wide text-zinc-100 uppercase">Node Explorer</h2>
            <p class="text-[10px] text-zinc-500 mt-1">Direct Shared Memory Access</p>
          </div>
          <div class="bg-blue-500/10 text-blue-400 text-[8px] font-bold px-1.5 py-0.5 rounded border border-blue-500/20">
            FFI-LOCKED
          </div>
        </div>

        <NodeProperties :nodeId="selectedNodeId" :isLoading="isLoadingNode" />

        <div class="mt-auto p-4 border-t border-zinc-800 bg-black/40 space-y-3">
          <div class="flex justify-between items-center px-1">
             <span class="text-[9px] text-zinc-600 uppercase font-bold">Auto-Sync</span>
             <div class="w-6 h-3 bg-emerald-900/50 rounded-full relative border border-emerald-500/30">
                <div class="absolute right-0.5 top-0.5 w-1.5 h-1.5 bg-emerald-400 rounded-full"></div>
             </div>
          </div>
          <button class="w-full py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white text-[10px] font-black rounded-md transition-all duration-300 uppercase tracking-widest shadow-lg shadow-emerald-900/10">
            Export Delta Log
          </button>
        </div>
      </aside>
    </transition>
  </div>
</template>

<style>
body, html {
  margin: 0;
  padding: 0;
  background: #000;
  overflow: hidden;
}

.transition-all {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
