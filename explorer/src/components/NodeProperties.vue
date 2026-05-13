<script setup lang="ts">
import { ref } from 'vue'

interface Property {
  key: string
  value: string | number
  group: 'LOGIC' | 'MEMORY' | 'ACTION'
}

const props = defineProps<{
  nodeId?: string
  isLoading: boolean
}>()

const properties = ref<Property[]>([
  { key: 'TypeID', value: 42, group: 'LOGIC' },
  { key: 'Weight', value: 0.985, group: 'LOGIC' },
  { key: 'LastUpdated', value: '2026-05-12T19:24', group: 'MEMORY' },
  { key: 'ActiveThreads', value: 16, group: 'ACTION' }
])
</script>

<template>
  <div class="p-6 space-y-6">
    <!-- Skeleton State -->
    <div v-if="isLoading" class="space-y-6">
      <div v-for="i in 3" :key="i" class="space-y-2">
        <div class="h-2 w-1/4 bg-zinc-800 rounded animate-pulse"></div>
        <div class="h-10 w-full bg-zinc-900 border border-zinc-800 rounded-lg animate-pulse"></div>
      </div>
    </div>

    <!-- Loaded State -->
    <div v-else class="space-y-6">
      <header>
        <h3 class="text-xs font-mono text-zinc-500 uppercase">Selected Node</h3>
        <p class="text-lg font-bold text-white tracking-tight">{{ nodeId || 'None Selected' }}</p>
      </header>

      <div class="space-y-4">
        <div v-for="prop in properties" :key="prop.key" class="group bg-zinc-900/50 border border-zinc-800 p-3 rounded-lg hover:border-emerald-500/50 transition-all duration-300">
          <div class="flex justify-between items-center">
            <span class="text-[10px] font-mono text-zinc-500 group-hover:text-emerald-400 transition-colors">{{ prop.key }}</span>
            <span :class="{
              'text-[8px] px-2 py-0.5 rounded-full font-bold': true,
              'bg-blue-900/30 text-blue-400': prop.group === 'LOGIC',
              'bg-purple-900/30 text-purple-400': prop.group === 'MEMORY',
              'bg-emerald-900/30 text-emerald-400': prop.group === 'ACTION'
            }">{{ prop.group }}</span>
          </div>
          <p class="mt-1 text-sm font-semibold text-zinc-200">{{ prop.value }}</p>
        </div>
      </div>

      <div class="pt-6 border-t border-zinc-800/50">
        <div class="flex items-center gap-2 mb-3">
          <div class="w-1.5 h-1.5 rounded-full bg-blue-500 shadow-[0_0_8px_#3b82f6]"></div>
          <span class="text-[10px] font-mono text-zinc-400 uppercase tracking-widest">Logic Flow</span>
        </div>
        <div class="h-1 w-full bg-zinc-900 rounded-full overflow-hidden">
          <div class="h-full bg-blue-500 w-3/4 shadow-[0_0_10px_#3b82f6]"></div>
        </div>
      </div>
    </div>
  </div>
</template>
