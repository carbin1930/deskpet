import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { SystemStats } from '../types/stats'

/** 订阅后端定时推送的系统状态；首屏先同步拉一次。 */
export function useStats() {
  const stats = ref<SystemStats | null>(null)
  let unlisten: (() => void) | null = null

  onMounted(async () => {
    try {
      stats.value = await invoke<SystemStats>('get_stats')
    } catch (e) {
      console.error('get_stats failed', e)
    }
    unlisten = await listen<SystemStats>('stats://tick', (event) => {
      stats.value = event.payload
    })
  })

  onUnmounted(() => {
    unlisten?.()
  })

  return { stats }
}
