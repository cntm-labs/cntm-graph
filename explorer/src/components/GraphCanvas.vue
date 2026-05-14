<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { GraphRenderer } from '@/core/renderer';

const canvasRef = ref<HTMLCanvasElement | null>(null);
let renderer: GraphRenderer | null = null;

onMounted(async () => {
  if (canvasRef.value) {
    renderer = new GraphRenderer(canvasRef.value);
    const success = await renderer.init();
    if (success) {
      renderer.start();
    }
  }
});
</script>

<template>
  <div class="relative w-full h-full bg-[#050505] overflow-hidden">
    <canvas ref="canvasRef" class="w-full h-full block"></canvas>
    <div class="absolute top-4 right-4 pointer-events-none">
      <div class="bg-black/80 border border-emerald-500/50 px-3 py-1 rounded-full text-[10px] text-emerald-400 font-mono tracking-widest uppercase shadow-[0_0_15px_rgba(16,185,129,0.2)]">
        WebGPU Active
      </div>
    </div>
  </div>
</template>

<style scoped>
canvas {
  image-rendering: pixelated;
}
</style>
