<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
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

let unlistenDetached: (() => void) | null = null

onMounted(async () => {
  try {
    const edge = await invoke<Edge | null>('is_docked')
    docked.value = edge
    // 进程重启后 Rust 侧默认未展开；若窗口尺寸异常，视为探头态。
    expanded.value = false
  } catch (e) {
    console.error('is_docked failed', e)
  }
  // 用户把探头小窗拖离边缘后，Rust 端已恢复完整尺寸，这里同步 UI 状态。
  try {
    unlistenDetached = await listen('dock://detached', () => {
      docked.value = null
      expanded.value = false
      menuOpen.value = false
    })
  } catch (e) {
    console.error('listen detached failed', e)
  }
})

onUnmounted(() => unlistenDetached?.())

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

    <!-- 探头态：两只前爪扒在边缘上，交替抓挠 -->
    <div v-if="peeking" class="peek-edge" aria-hidden="true">
      <div class="edge-bar"></div>
      <svg class="paw paw-left" viewBox="0 0 48 30">
        <rect x="16" y="12" width="16" height="18" rx="6" fill="#d9a54a" />
        <ellipse cx="24" cy="17" rx="13" ry="10" fill="#f7e9c9" />
        <circle cx="13" cy="10" r="4.5" fill="#f7e9c9" />
        <circle cx="24" cy="7" r="5" fill="#f7e9c9" />
        <circle cx="35" cy="10" r="4.5" fill="#f7e9c9" />
        <ellipse cx="24" cy="19" rx="5" ry="4" fill="#e8899a" opacity=".85" />
      </svg>
      <svg class="paw paw-right" viewBox="0 0 48 30">
        <rect x="16" y="12" width="16" height="18" rx="6" fill="#d9a54a" />
        <ellipse cx="24" cy="17" rx="13" ry="10" fill="#f7e9c9" />
        <circle cx="13" cy="10" r="4.5" fill="#f7e9c9" />
        <circle cx="24" cy="7" r="5" fill="#f7e9c9" />
        <circle cx="35" cy="10" r="4.5" fill="#f7e9c9" />
        <ellipse cx="24" cy="19" rx="5" ry="4" fill="#e8899a" opacity=".85" />
      </svg>
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
/* 把 170 的宠物放大并上移，让头/耳朵填满探头小窗，身体下缘留给扒边爪子 */
.pet-window.peeking .pet {
  transform: translateY(-26px) scale(1.12);
  transform-origin: center top;
}

/* 探头态扒边：底部一条屏幕边缘，两只前爪搭在上面交替抓挠 */
.peek-edge {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 2;
}
.peek-edge .edge-bar {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 18px;
  background: linear-gradient(180deg, rgba(120, 134, 156, 0.9), rgba(70, 82, 100, 0.95));
  border-top: 2px solid rgba(203, 213, 225, 0.55);
  box-shadow: 0 -3px 10px rgba(0, 0, 0, 0.25);
}
.peek-edge .paw {
  position: absolute;
  bottom: 12px;
  width: 38px;
  height: 24px;
}
.peek-edge .paw-left {
  left: 20px;
  animation: paw-climb 1.1s ease-in-out infinite;
}
.peek-edge .paw-right {
  right: 20px;
  animation: paw-climb 1.1s ease-in-out infinite 0.55s;
}
@keyframes paw-climb {
  0%, 100% { transform: translateY(0) rotate(-4deg); }
  50% { transform: translateY(5px) rotate(4deg); }
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
