# 桌面电子宠物 · 第一期方案设计

> 目标形态：常驻桌面的透明小宠物，实时反映本机 CPU / 内存 / 磁盘 / 网络状态。
> 技术栈：Tauri 2 + Vue 3 + Rust(sysinfo)。平台：Windows + macOS。

---

## 1. 范围界定

### 第一期做（In Scope）
- 透明、无边框、始终置顶、可拖拽的宠物窗口
- 四项指标采集：CPU 使用率、内存占用、磁盘占用、网络上下行速率
- 1–2s 轮询，前端以仪表/数值/宠物表情呈现
- 宠物状态机：指标越界时切换表情/动画（闲适 → 忙碌 → 告警）
- 托盘图标：显示/隐藏、退出
- Windows + macOS 双平台适配

### 第一期不做（Out of Scope）
- 宠物养成、互动、音效
- 告警弹窗/通知推送（状态机内部体现即可）
- 多宠物、主题商店、设置面板
- 移动端

---

## 2. 技术选型与跨平台要点

| 层 | 选型 | 理由 |
|---|---|---|
| 应用框架 | Tauri 2 | 复用系统 WebView，常驻内存低，原生支持透明置顶 |
| 前端 | Vue 3 + TypeScript + Vite | 轻量、动画生态好，打包产物小 |
| 采集 | Rust `sysinfo` crate | 一个 crate 覆盖四项指标，稳定跨平台 |
| 通信 | Tauri `invoke` + `event` | 前端订阅后端定时推送 |

### 选型决策依据（为什么不选更省内存的原生方案）

评估过 Slint / egui / iced 等无 WebView 的原生渲染方案（常驻约 15–30MB，低于 Tauri 的约 55MB），**最终仍选 Tauri**，原因：

- 用户明确要求宠物后期支持**炫酷特效**与**自定义形象**。自定义形象本质是运行时加载用户提供的富媒体资源（位图 / SVG / Lottie / 序列帧），Web 生态有成熟解码、合成、滤镜、GPU 动画能力，原生轻量 GUI 需自造整套轮子。
- WebView 的约 55MB 是**固定成本**，与宠物特效复杂度基本无关。加特效只增加少量纹理缓存，不会线性膨胀；换 Slint 省下的约 35MB，代价是失去整个动画生态，不划算。
- 若目标机器为 16GB 内存，55MB 占比约 0.3%，实际无感。

> 结论：Tauri 2 + Web 渲染是"表现力优先"下的正确取舍。第一期的架构预留见第 11 节。

### 跨平台适配清单（重点）

| 项 | Windows | macOS | 处理方式 |
|---|---|---|---|
| WebView | WebView2（系统自带/需引导安装） | WKWebView（系统自带） | 打包时检测 WebView2，缺失则引导安装 |
| 透明窗口 | 原生支持 | **需开启 `macos-private-api`** | `Cargo.toml` 的 tauri feature 加 `macos-private-api` |
| 置顶行为 | `alwaysOnTop` | `alwaysOnTop` + 可能需 `setAlwaysOnTop` 显式调用 | 统一用 Tauri API，macOS 额外验证层级 |
| 图标格式 | `.ico` | `.icns` | 两套装图标，配置里分别指定 |
| 点击穿透 | Windows 支持良好 | 需透明窗口 feature | 宠物本体可交互区与透明拖拽区分离 |
| 签名/公证 | 无强制 | 分发需签名+公证 | 第一期仅本地构建，暂不处理 |

---

## 3. 目录结构

```
deskpet/
├── index.html
├── package.json
├── vite.config.ts
├── tsconfig.json
├── src/                          # Vue 前端
│   ├── main.ts
│   ├── App.vue
│   ├── components/
│   │   ├── Pet.vue               # 宠物本体（SVG + 动画）
│   │   ├── MetricPanel.vue       # 四指标仪表盘
│   │   └── TrayMenu.vue          # （逻辑在 Rust，前端仅占位）
│   ├── composables/
│   │   ├── useStats.ts           # 监听 stats 事件，维护指标快照
│   │   └── usePetState.ts        # 状态机：指标 → 情绪
│   └── types/
│       └── stats.ts              # 与 Rust 对齐的 TS 类型
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json           # 窗口透明/置顶/尺寸/图标
│   ├── capabilities/
│   │   └── default.json          # 权限声明
│   ├── icons/                    # .ico / .icns / .png
│   └── src/
│       ├── main.rs               # 入口
│       ├── lib.rs                # Tauri builder / 插件注册
│       ├── model.rs              # SystemStats 结构体
│       ├── monitor.rs            # sysinfo 采集逻辑
│       └── commands.rs           # invoke 命令（含采样调度）
└── docs/
    └── DESIGN.md
```

---

## 4. 数据契约

### 4.1 Rust 数据结构（src-tauri/src/model.rs）

```rust
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub cpu_usage: f32,        // 0..100
    pub mem_used: u64,         // bytes
    pub mem_total: u64,
    pub mem_usage: f32,        // 0..100
    pub disk_used: u64,
    pub disk_total: u64,
    pub disk_usage: f32,
    pub net_rx: u64,           // bytes/s
    pub net_tx: u64,
    pub sampled_at: i64,       // unix millis
    pub status: PetStatus,     // 由后端算好，前端直接用
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PetStatus {
    Idle,   // 闲适
    Busy,   // 忙碌
    Alert,  // 告警
}
```

### 4.2 TypeScript 对应（src/types/stats.ts）

```ts
export interface SystemStats {
  cpuUsage: number
  memUsed: number
  memTotal: number
  memUsage: number
  diskUsed: number
  diskTotal: number
  diskUsage: number
  netRx: number
  netTx: number
  sampledAt: number
  status: 'idle' | 'busy' | 'alert'
}
```

> 约定：Rust 用 `#[serde(rename_all = "camelCase")]`，前后端字段自动对齐，避免手写映射。

---

## 5. IPC 接口设计

### 5.1 事件推送（Rust → 前端）
采样任务每 1.5s 推送一次全量快照，前端只需订阅。

```rust
// 后端
app.emit("stats://tick", &stats).unwrap();
```

```ts
// 前端 composables/useStats.ts
import { listen } from '@tauri-apps/api/event'
import type { SystemStats } from '../types/stats'

listen<SystemStats>('stats://tick', (event) => {
  stats.value = event.payload
})
```

### 5.2 命令调用（前端 → Rust）

| 命令 | 入参 | 返回 | 用途 |
|---|---|---|---|
| `get_stats` | 无 | `SystemStats` | 首屏立即取一次，避免等 1.5s |
| `set_interval_ms` | `ms: u64` | `()` | 调整采样频率（预留，第一期不暴露 UI） |

> 事件推送为主、`invoke` 为辅。后端用 Tauri 的 `AppHandle` 在独立线程中定时采样，不阻塞主线程。

---

## 6. 采集层设计（src-tauri/src/monitor.rs）

### 6.1 采样策略
- 全局一个 `Sys` 实例（`System::new_all()`），复用而非反复 new，降低开销。
- CPU 使用率：`sys.global_cpu_info().cpu_usage()`。注意 sysinfo 首次调用需先 `refresh_cpu()` 预热一次，否则首帧为 0。
- 内存：`sys.used_memory()` / `sys.total_memory()`。
- 磁盘：取系统主盘（Windows 为 `C:\`，macOS 为 `/`），用 `Disks::new_with_refreshed_list()` 后按挂载点筛选；对主盘调用 `refresh()`。
- 网络：`Networks::new_with_refreshed_list()`，累加所有非 loopback 网卡的 `received()` / `transmitted()`，与上次采样做**差值 / 时间间隔**得速率；首次采样速率记 0。

### 6.2 交叉平台的实现隔离
```rust
#[cfg(target_os = "windows")]
fn root_mount() -> &'static str { "C:\\" }

#[cfg(target_os = "macos")]
fn root_mount() -> &'static str { "/" }
```
其余采集逻辑平台无关，共用一套代码。

### 6.3 性能约束
- 采样间隔 ≥ 1000ms，避免采集本身成为负载。
- 磁盘列表只在初始化时拉取一次，之后只 refresh 主盘，避免每次遍历所有分区。
- 前端渲染做节流，收到事件后合并到下一次 `requestAnimationFrame`。

---

## 7. 宠物状态机（src-tauri/src/monitor.rs 计算 + 前端消费）

### 7.1 分级规则（取四项中的最大"压力值"）

| 状态 | 触发条件 | 宠物表现 | 颜色 |
|---|---|---|---|
| `idle` 闲适 | 所有指标 < 60% | 微动、眨眼、微笑 | 蓝/绿 |
| `busy` 忙碌 | 任一指标 60%–84% | 动作加快、冒汗 | 琥珀 |
| `alert` 告警 | 任一指标 ≥ 85% | 红色、皱眉、抖动 | 红 |

```rust
fn pressure(stats: &SystemStats) -> f32 {
    stats.cpu_usage
        .max(stats.mem_usage)
        .max(stats.disk_usage)
        .max(net_pressure(stats))   // 网速按阈值归一，见 7.2
}
```

### 7.2 网络指标的归一
网络没有"总量百分比"，按带宽阈值映射：
```
net_rx 或 net_tx >= 10 MB/s  → 视为 100% 压力
线性映射：pressure = min(rate_bps / 10_000_000, 1.0) * 100
```
阈值后续可在配置中调整。

### 7.3 抖动抑制
状态切换加**迟滞 + 确认次数**：连续 2 次采样满足新状态才切换，避免临界值附近频繁跳变。

---

## 8. 窗口与交互

### 8.1 tauri.conf.json 关键配置
```json
{
  "app": { "windows": [{
    "label": "main",
    "width": 220,
    "height": 240,
    "decorations": false,
    "transparent": true,
    "alwaysOnTop": true,
    "resizable": false,
    "skipTaskbar": true,
    "center": true
  }]},
  "bundle": {
    "icon": ["icons/32x32.png", "icons/icon.ico", "icons/icon.icns"]
  }
}
```

### 8.2 拖拽实现
宠物根节点加 `data-tauri-drag-region`，透明区域自动穿透拖拽，交互按钮（如关闭）单独排除。

### 8.3 托盘
用 Tauri 2 的 `tray` 插件，菜单项：显示/隐藏、退出。窗口关闭按钮隐藏窗口而非退出进程。

---

## 9. 构建与验证

### 9.1 关键依赖
```toml
# Cargo.toml
[dependencies]
tauri = { version = "2", features = ["macos-private-api", "tray-icon"] }
sysinfo = "0.33"
serde = { version = "1", features = ["derive"] }
```

### 9.2 开发命令
```bash
npm install
npm run tauri dev      # 开发热更新
npm run tauri build    # 生产打包
```

### 9.3 验证清单
- 两平台窗口透明、置顶、拖拽正常
- 四项指标数值与系统任务管理器/活动监视器大致吻合
- 采样 5 分钟，宠物进程 CPU / 内存占用稳定在低位
- 临界值附近状态不抖动

---

## 11. 面向自定义宠物的架构预留（第一期就要做）

用户明确要求后期支持炫酷特效与自定义形象。第一期虽只内置一套宠物，但以下四点**必须在第一期落地**，否则第二期会大改：

### 11.1 资源抽象层：宠物 = 一组带键名的资源 + 一份清单

不把形象写死在组件里，而是抽象为可替换的"资源包"：

```
pet-pack/
├── manifest.json          # 元信息 + 状态到动画的映射
├── idle.svg / idle.json   # 闲适态（SVG 或 Lottie）
├── busy.svg / busy.json
├── alert.svg / alert.json
└── sprite.png             # 可选：序列帧
```

manifest 示例：
```json
{
  "id": "cat-default",
  "name": "默认小猫",
  "version": 1,
  "animations": {
    "idle":  { "src": "idle.json", "type": "lottie", "loop": true },
    "busy":  { "src": "busy.json", "type": "lottie", "loop": true },
    "alert": { "src": "alert.json", "type": "lottie", "loop": true }
  }
}
```

### 11.2 渲染抽象：状态 → 动画的映射与后端状态机解耦

后端只输出抽象状态 `idle/busy/alert`（见第 7 节），前端维护 `status → 动画资源` 的映射表。第二期换形象只需替换资源包和映射，后端采集与状态机代码零改动。

```
SystemStats.status (后端)  →  PetRenderer.play(status)  →  当前资源包的对应动画
```

### 11.3 技术选型配套
- **动画运行时**：第一期用 SVG + CSS 过渡足够；渲染层封装成 `PetRenderer` 接口，第二期可平滑替换为 Lottie-web（矢量特效）或 PixiJS（粒子/滤镜），组件调用方不变。
- **资源加载**：预留 Tauri fs 插件权限，资源包放应用数据目录，路径通过 IPC 传入。
- **安全**：自定义资源需校验类型白名单（svg/lottie/png/webp），拒绝脚本与远程 URL，防 XSS。

### 11.4 第一期验收项（避免返工）
- [ ] 宠物形象通过资源包 + manifest 加载，不硬编码在 Vue 模板中
- [ ] `PetRenderer` 为独立模块，只暴露 `play(status)` 接口
- [ ] 状态机输出与具体动画实现无耦合

---

## 12. 后续演进（第二期候选）

- 自定义形象 / 主题商店（基于第 11 节资源包协议）
- 炫酷特效：Lottie 动画 / PixiJS 粒子 / 毛玻璃
- 宠物养成与互动（点击、投喂、成长值）
- 达到阈值时的系统通知
- 历史曲线与小组件
- 采样频率、阈值、网速基准做成设置面板

---

*文档版本：v1.1 · 2026-11-20*
