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
- 边缘停靠：趴在屏幕左/右/上边缘，只露一条边，鼠标悬停时整窗滑出

### 操作

- **右键**：呼出菜单（停靠到左边/右边/顶部、离开边缘、隐藏窗口、退出）
- **双击**宠物：快速趴到右边缘 / 脱离边缘
- **悬停**停靠后露出的边缘：窗口整窗滑出，鼠标移开自动缩回
- **拖拽**宠物可移动窗口；托盘图标可显示 / 隐藏 / 退出

### 运行

需要 Node 18+、Rust 1.77+，以及对应平台的原生工具链：

- **Windows**：MSVC Build Tools（含 `link.exe`），或 MinGW-w64（含 `dlltool.exe`）
- **macOS**：Xcode Command Line Tools

```bash
npm install
npm run tauri dev                   # 开发模式，热更新（首次会编译 Rust，稍慢）
npm run tauri -- build --no-bundle  # 生成独立 exe，跳过安装包
```

产物位于 `src-tauri/target/release/deskpet.exe`。

**本机一键脚本（无 VS Build Tools 时）**：仓库在 `.tools/` 附带了 GNU 工具链，但它是 gitignore 的、且不会自动进 PATH。若直接跑 `cargo check` / `tauri dev` 报 `dlltool.exe: program not found`，用这两个脚本即可（自动配 PATH + linker，并规避 Defender 锁文件）：

```bash
# 一键构建并运行：增量编译出 exe 后直接弹出宠物窗口（托盘图标可退出）
powershell -ExecutionPolicy Bypass -File scripts\dev-local.ps1

# 仅生成独立 exe，不运行
powershell -ExecutionPolicy Bypass -File scripts\build-local.ps1
```

> `dev-local.ps1` 内部复用 `build-local.ps1` 的工具链配置（PATH + linker），并强制单线程编译、带重试，规避本机 GNU 工具链的 dlltool / dll_imports.lib / Defender 锁文件等坑。首次全量编译较慢，之后走缓存很快。
>
> `.tools/` 为本地工具链，换电脑后需重新准备（见 `scripts/build-local.ps1` 顶部注释）。有 VS Build Tools 的机器可忽略这些脚本，直接用标准命令。

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
- Edge docking: perch on the left/right/top edge, revealing only a thin strip; hover to slide the full window out

### Controls

- **Right-click**: open the menu (dock to left/right/top, undock, hide, quit)
- **Double-click** the pet: quickly dock to the right edge / undock
- **Hover** the exposed edge while docked: the window slides out and retracts when the mouse leaves
- **Drag** the pet to move the window; use the tray icon to show / hide / quit

### Run it

Requires Node 18+, Rust 1.77+, and the native toolchain for your platform:

- **Windows**: MSVC Build Tools (provides `link.exe`), or MinGW-w64 (provides `dlltool.exe`)
- **macOS**: Xcode Command Line Tools

```bash
npm install
npm run tauri dev                   # dev with hot reload (first run compiles Rust)
npm run tauri -- build --no-bundle  # standalone exe, skip installer
```

The exe lands at `src-tauri/target/release/deskpet.exe`.

**One-click scripts for this machine (no VS Build Tools)**: the repo ships a GNU toolchain in `.tools/`, but it is gitignored and not auto-added to PATH. If a plain `cargo check` / `tauri dev` fails with `dlltool.exe: program not found`, use these scripts (they set PATH + linker and work around Defender file locks):

```bash
# One-click build + run: incrementally builds the exe, then launches the pet window
powershell -ExecutionPolicy Bypass -File scripts\dev-local.ps1

# Build standalone exe only (no launch)
powershell -ExecutionPolicy Bypass -File scripts\build-local.ps1
```

> `dev-local.ps1` reuses the toolchain setup in `build-local.ps1` (PATH + linker), forces single-threaded compilation and adds retries to work around this box's GNU-toolchain quirks (dlltool / dll_imports.lib / Defender file locks). The first full build is slow; later runs hit the cache and are fast.
>
> `.tools/` is a local toolchain; it must be re-prepared on a new machine (see the header of `scripts/build-local.ps1`). Machines with VS Build Tools can ignore these scripts and use the standard commands.

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
