<script setup lang="ts">
import { computed } from 'vue'
import Pet from './components/Pet.vue'
import MetricPanel from './components/MetricPanel.vue'
import { useStats } from './composables/useStats'
import type { PetStatus } from './types/stats'

const { stats } = useStats()

const status = computed<PetStatus>(() => stats.value?.status ?? 'idle')
</script>

<template>
  <div class="pet-window">
    <Pet :status="status" />
    <MetricPanel :stats="stats" />
  </div>
</template>

<style>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}
html,
body {
  background: transparent;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
}
#app {
  width: 220px;
  height: 260px;
}
.pet-window {
  width: 220px;
  height: 260px;
  display: flex;
  flex-direction: column;
  padding-top: 6px;
  user-select: none;
}
</style>
