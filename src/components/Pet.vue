<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import type { PetStatus } from '../types/stats'
import type { PetManifest, PetRenderer } from '../renderer/types'
import { CssPetRenderer } from '../renderer/PetRenderer'

const props = defineProps<{ status: PetStatus }>()

const layers: { state: PetStatus; src: string }[] = [
  { state: 'idle', src: '/packs/default/idle.svg' },
  { state: 'busy', src: '/packs/default/busy.svg' },
  { state: 'alert', src: '/packs/default/alert.svg' },
]

const host = ref<HTMLElement | null>(null)
let renderer: PetRenderer | null = null

onMounted(async () => {
  if (!host.value) return
  try {
    const res = await fetch('/packs/default/manifest.json')
    const manifest: PetManifest = await res.json()
    renderer = new CssPetRenderer(host.value, manifest)
    renderer.play(props.status)
  } catch (e) {
    console.error('load pet pack failed', e)
  }
})

watch(
  () => props.status,
  (s) => renderer?.play(s),
)

onUnmounted(() => renderer?.dispose?.())
</script>

<template>
  <div ref="host" class="pet" data-tauri-drag-region>
    <img
      v-for="layer in layers"
      :key="layer.state"
      class="layer"
      :class="{ active: status === layer.state }"
      :src="layer.src"
      :alt="layer.state"
    />
  </div>
</template>

<style scoped>
.pet {
  position: relative;
  width: 170px;
  height: 170px;
  margin: 0 auto;
  user-select: none;
}
.layer {
  position: absolute;
  inset: 0;
  width: 170px;
  height: 170px;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.35s ease;
}
.layer.active {
  opacity: 1;
}
</style>
