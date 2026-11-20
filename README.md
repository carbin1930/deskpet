# deskpet

常驻桌面的电子宠物，实时反映本机 CPU / 内存 / 磁盘 / 网络状态。第一期支持 Windows + macOS。

A desktop pet that lives on your screen and reflects your machine's CPU, memory, disk, and network status in real time. Phase 1 supports Windows + macOS.

> 语言 / Language: [中文](#中文) · [English](#english)

---

<a id="中文"></a>

## 中文

### 技术栈

- **Tauri 2** — 应用框架（透明、无边框、始终置顶）
- **Vue 3 + TypeScript + Vite** — 前端
- **Rust `sysinfo`** — 系统指标采集

架构与后续演进见 [`docs/DESIGN.md`](docs/DESIGN.md)。

### 功能

- 四项指标实时采集，1.5s 推送一次
- 宠物状态机：`idle`（闲适）/ `busy`（忙碌）/ `alert`（告警），取最大压力值并带迟滞防抖
- 透明置顶、可拖拽、托盘显示/隐藏/退出
- 宠物资源包协议（manifest + idle/busy/alert 动画），为第二期自定义形象预留

### 开发环境

需要 Node 18+、Rust 1.77+，以及对应平台的原生工具链：

- **Windows**：MSVC Build Tools（含 `link.exe`），或 MinGW-w64（含 `dlltool.exe`）
- **macOS**：Xcode Command Line Tools

```bash
npm install
npm run tauri dev      # 开发热更新
npm run tauri build    # 生产打包
```

仅验证前端：

```bash
npm run build
```

### 宠物资源包

宠物形象抽象为资源包，位于 `public/packs/<id>/`：

```
default/
├── manifest.json      # 状态 → 动画文件映射
├── idle.svg
├── busy.svg
└── alert.svg
```

`PetRenderer` 接口只暴露 `play(status)`。第二期可替换为 Lottie / PixiJS 渲染器，无需改动状态机。

---

<a id="english"></a>

## English

### Tech stack

- **Tauri 2** — application framework (transparent, borderless, always-on-top)
- **Vue 3 + TypeScript + Vite** — frontend
- **Rust `sysinfo`** — system metrics collection

See [`docs/DESIGN.md`](docs/DESIGN.md) for the architecture and roadmap.

### Features

- Real-time collection of four metrics, pushed every 1.5s
- Pet state machine: `idle` / `busy` / `alert`, driven by the peak pressure value with hysteresis to prevent flapping
- Transparent, always-on-top, draggable window; tray menu for show/hide and quit
- Pet resource pack protocol (manifest + idle/busy/alert animations), designed for custom skins in Phase 2

### Development setup

Requires Node 18+, Rust 1.77+, and the native toolchain for your platform:

- **Windows**: MSVC Build Tools (provides `link.exe`), or MinGW-w64 (provides `dlltool.exe`)
- **macOS**: Xcode Command Line Tools

```bash
npm install
npm run tauri dev      # dev with hot reload
npm run tauri build    # production build
```

Frontend-only validation:

```bash
npm run build
```

### Pet resource packs

Pet appearances are abstracted as resource packs under `public/packs/<id>/`:

```
default/
├── manifest.json      # status → animation file mapping
├── idle.svg
├── busy.svg
└── alert.svg
```

The `PetRenderer` interface exposes only `play(status)`. In Phase 2 the renderer can be swapped for Lottie / PixiJS without touching the state machine.
