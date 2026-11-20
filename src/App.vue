<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Pet from './components/Pet.vue'
import MetricPanel from './components/MetricPanel.vue'
import { useStats } from './composables/useStats'
import type { PetStatus } from './types/stats'

const { stats } = useStats()
const status = computed<PetStatus>(() => stats.value?.status ?? 'idle')

type Edge = 'left' | 'right' | 'top'
const docked = ref<Edge | null>(null)
const menuOpen = ref(false)
const menuPos = ref({ x: 0, y: 0 })

onMounted(async () => {
  try {
    docked.value = await invoke<Edge | null>('is_docked')
  } catch (e) {
    console.error('is_docked failed', e)
  }
})

function openMenu(e: MouseEvent) {
  e.preventDefault()
  e.stopPropagation()
  menuPos.value = {
    x: Math.min(e.offsetX, 220 - 156),
    y: Math.min(e.offsetY, 260 - 210),
  }
  menuOpen.value = true
}

function closeMenu() {
  menuOpen.value = false
}

/** 停靠后只露一条边，鼠标移进去整窗滑出。 */
async function expand() {
  if (!docked.value) return
  try { await invoke('expand_dock') } catch (e) { console.error(e) }
}

async function collapse() {
  if (!docked.value) return
  try { await invoke('collapse_dock') } catch (e) { console.error(e) }
}

async function dockTo(edge: Edge) {
  closeMenu()
  try {
    await invoke('dock', { edge })
    docked.value = edge
  } catch (e) { console.error('dock failed', e) }
}

async function undock() {
  closeMenu()
  try {
    await invoke('undock')
    docked.value = null
  } catch (e) { console.error('undock failed', e) }
}

/** 双击：已停靠则脱离，未停靠则默认趴到右边缘。 */
function toggleDock() {
  if (docked.value) void undock()
  else void dockTo('right')
}

async function hide() {
  closeMenu()
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().hide()
  } catch (e) { console.error(e) }
}

async function quit() {
  closeMenu()
  try { await invoke('quit_app') } catch (e) { console.error(e) }
}

function onRootClick() {
  if (menuOpen.value) closeMenu()
}
</script>

<template>
  <div
    class="pet-window"
    @contextmenu="openMenu"
    @dblclick="toggleDock"
    @mouseenter="expand"
    @mouseleave="collapse"
    @click="onRootClick"
  >
    <Pet :status="status" />
    <MetricPanel :stats="stats" />

    <div
      v-if="menuOpen"
      class="ctx-menu"
      :style="{ left: `${menuPos.x}px`, top: `${menuPos.y}px` }"
      @click.stop
      @dblclick.stop
      @contextmenu.stop.prevent
    >
      <div class="menu-title">趴在边缘</div>
      <button @click="dockTo('left')">停靠 · 左边</button>
      <button @click="dockTo('right')">停靠 · 右边</button>
      <button @click="dockTo('top')">停靠 · 顶部</button>
      <button v-if="docked" @click="undock">离开边缘</button>
      <div class="menu-sep"></div>
      <button @click="hide">隐藏窗口</button>
      <button @click="quit">退出</button>
    </div>
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
  position: relative;
  width: 220px;
  height: 260px;
  display: flex;
  flex-direction: column;
  padding-top: 4px;
  user-select: none;
}

/* 自定义右键菜单，替代 WebView 原生菜单 */
.ctx-menu {
  position: absolute;
  z-index: 10;
  width: 150px;
  padding: 5px;
  border-radius: 10px;
  background: rgba(20, 27, 38, 0.95);
  border: 1px solid rgba(148, 163, 184, 0.25);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
}
.menu-title {
  padding: 4px 8px 5px;
  font-size: 10px;
  color: #64748b;
  letter-spacing: 0.5px;
}
.ctx-menu button {
  display: block;
  width: 100%;
  padding: 7px 8px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: #e2e8f0;
  font-size: 12px;
  text-align: left;
  cursor: pointer;
}
.ctx-menu button:hover {
  background: rgba(96, 165, 250, 0.18);
}
.menu-sep {
  height: 1px;
  margin: 5px 4px;
  background: rgba(148, 163, 184, 0.2);
}
</style>
