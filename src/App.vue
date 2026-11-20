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
const expanded = ref(false)
const menuOpen = ref(false)
const menuPos = ref({ x: 0, y: 0 })

/** 停靠但未展开 = 探头小窗（窗口视口 118x118）。 */
const peeking = computed(() => docked.value !== null && !expanded.value)

onMounted(async () => {
  try {
    const edge = await invoke<Edge | null>('is_docked')
    docked.value = edge
    // 进程重启后 Rust 侧默认未展开；若窗口尺寸异常，视为探头态。
    expanded.value = false
  } catch (e) {
    console.error('is_docked failed', e)
  }
})

function openMenu(e: MouseEvent) {
  e.preventDefault()
  e.stopPropagation()
  if (peeking.value) {
    // 探头态视口很小，菜单贴左上角并紧凑显示。
    menuPos.value = { x: 2, y: 2 }
  } else {
    menuPos.value = {
      x: Math.min(e.offsetX, 220 - 156),
      y: Math.min(e.offsetY, 260 - 210),
    }
  }
  menuOpen.value = true
}

function closeMenu() {
  menuOpen.value = false
}

/** 停靠后只露一个探头，鼠标移进去整窗展开。 */
async function expand() {
  if (!docked.value || expanded.value) return
  try {
    await invoke('expand_dock')
    expanded.value = true
  } catch (e) { console.error(e) }
}

async function collapse() {
  if (!docked.value || !expanded.value) return
  try {
    await invoke('collapse_dock')
    expanded.value = false
  } catch (e) { console.error(e) }
}

async function dockTo(edge: Edge) {
  closeMenu()
  try {
    await invoke('dock', { edge })
    docked.value = edge
    expanded.value = false
  } catch (e) { console.error('dock failed', e) }
}

async function undock() {
  closeMenu()
  try {
    await invoke('undock')
    docked.value = null
    expanded.value = false
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
    :class="{ peeking }"
    @contextmenu="openMenu"
    @dblclick="toggleDock"
    @mouseenter="expand"
    @mouseleave="collapse"
    @click="onRootClick"
  >
    <div class="stage">
      <Pet :status="status" />
    </div>
    <div class="metrics">
      <MetricPanel :stats="stats" />
    </div>

    <div
      v-if="menuOpen"
      class="ctx-menu"
      :class="{ compact: peeking }"
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
  width: 100%;
  height: 100%;
}
.pet-window {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  user-select: none;
}
.stage {
  flex: 0 0 auto;
}
.metrics {
  flex: 0 0 auto;
}

/* ===== 探头态（停靠、未展开）：视口 118x118，只显示宠物头部特写 ===== */
.pet-window.peeking {
  width: 118px;
  height: 118px;
  overflow: hidden;
}
.pet-window.peeking .metrics {
  display: none;
}
.pet-window.peeking .stage {
  width: 118px;
  height: 118px;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  overflow: hidden;
}
/* 把 170 的宠物放大并上移，让头/耳朵填满探头小窗 */
.pet-window.peeking .pet {
  transform: translateY(-30px) scale(1.18);
  transform-origin: center top;
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
.ctx-menu.compact {
  width: 114px;
  padding: 3px;
}
.ctx-menu.compact button {
  padding: 5px 6px;
  font-size: 11px;
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
