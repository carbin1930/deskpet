<script setup lang="ts">
import { computed } from 'vue'
import type { SystemStats } from '../types/stats'

const props = defineProps<{ stats: SystemStats | null }>()

function mbps(bytes: number): string {
  if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB/s`
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB/s`
  return `${bytes} B/s`
}

const bars = computed(() => {
  const s = props.stats
  if (!s) return []
  return [
    { label: 'CPU', value: s.cpuUsage, text: `${s.cpuUsage.toFixed(0)}%` },
    { label: '内存', value: s.memUsage, text: `${s.memUsage.toFixed(0)}%` },
    { label: '磁盘', value: s.diskUsage, text: `${s.diskUsage.toFixed(0)}%` },
    {
      label: '网络',
      value: Math.min(((Math.max(s.netRx, s.netTx) / (10 * 1024 * 1024)) * 100), 100),
      text: `↓${mbps(s.netRx)} ↑${mbps(s.netTx)}`,
    },
  ]
})

function barColor(v: number): string {
  if (v >= 85) return '#ef4444'
  if (v >= 60) return '#f59e0b'
  return '#60a5fa'
}
</script>

<template>
  <div class="panel">
    <div v-for="bar in bars" :key="bar.label" class="row">
      <span class="label">{{ bar.label }}</span>
      <div class="track">
        <div class="fill" :style="{ width: `${bar.value}%`, background: barColor(bar.value) }" />
      </div>
      <span class="value">{{ bar.text }}</span>
    </div>
    <div v-if="!stats" class="loading">采集中…</div>
  </div>
</template>

<style scoped>
.panel {
  padding: 0 14px 12px;
  display: flex;
  flex-direction: column;
  gap: 7px;
}
.row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.label {
  width: 32px;
  font-size: 11px;
  color: #94a3b8;
  flex-shrink: 0;
}
.track {
  flex: 1;
  height: 6px;
  border-radius: 3px;
  background: rgba(148, 163, 184, 0.2);
  overflow: hidden;
}
.fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.4s ease, background 0.4s ease;
}
.value {
  width: 84px;
  text-align: right;
  font-size: 10px;
  color: #cbd5e1;
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}
.loading {
  font-size: 11px;
  color: #64748b;
  text-align: center;
}
</style>
