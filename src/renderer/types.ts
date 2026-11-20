import type { PetStatus } from '../types/stats'

/** 单个状态对应的动画描述。 */
export interface AnimationDef {
  src: string
  type: 'svg' | 'lottie' | 'sequence' | 'css'
  loop: boolean
  fps?: number
}

/** 资源包清单。第二期自定义形象沿用同一协议。 */
export interface PetManifest {
  id: string
  name: string
  version: number
  animations: Record<PetStatus, AnimationDef>
}

/** 渲染器对外只暴露状态播放能力，与状态机/采集解耦。 */
export interface PetRenderer {
  play(status: PetStatus): void
  dispose?(): void
}
