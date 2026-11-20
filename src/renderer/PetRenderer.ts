import type { PetManifest, PetRenderer, AnimationDef } from './types'
import type { PetStatus } from '../types/stats'

/**
 * 基于 CSS 类切换的渲染器（第一期实现）。
 * 第二期可替换为 LottieRenderer / PixiRenderer，只要实现同一 PetRenderer 接口，
 * 调用方（Pet.vue）无需改动。
 */
export class CssPetRenderer implements PetRenderer {
  private host: HTMLElement
  private manifest: PetManifest
  private current: PetStatus | null = null

  constructor(host: HTMLElement, manifest: PetManifest) {
    this.host = host
    this.manifest = manifest
  }

  play(status: PetStatus): void {
    if (status === this.current) return
    this.current = status

    const def: AnimationDef | undefined = this.manifest.animations[status]
    // 状态名直接作为 data 属性暴露给 CSS，动画资源路径写入 style 变量供后续渲染器读取
    this.host.dataset.petStatus = status
    if (def) {
      this.host.style.setProperty('--pet-anim-src', `url(${def.src})`)
      this.host.style.setProperty('--pet-anim-duration', def.fps ? `${(1000 / def.fps).toFixed(0)}ms` : '600ms')
    }
  }

  dispose(): void {
    this.host.removeAttribute('data-pet-status')
  }
}
