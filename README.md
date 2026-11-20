# deskpet

常驻桌面的电子宠物，实时反映本机 CPU / 内存 / 磁盘 / 网络状态。第一期支持 Windows + macOS。

## 技术栈

- **Tauri 2** — 应用框架（透明、无边框、始终置顶）
- **Vue 3 + TypeScript + Vite** — 前端
- **Rust `sysinfo`** — 系统指标采集

架构与后续演进见 [`docs/DESIGN.md`](docs/DESIGN.md)。

## 功能

- 四项指标实时采集，1.5s 推送一次
- 宠物状态机：`idle`（闲适）/ `busy`（忙碌）/ `alert`（告警），带迟滞防抖
- 透明置顶、可拖拽、托盘显示/隐藏/退出
- 宠物资源包协议（manifest + idle/busy/alert 动画），为第二期自定义形象预留

## 开发环境

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

## 宠物资源包

宠物形象抽象为资源包，位于 `public/packs/<id>/`：

```
default/
├── manifest.json      # 状态 → 动画文件映射
├── idle.svg
├── busy.svg
└── alert.svg
```

`PetRenderer` 接口只暴露 `play(status)`。第二期可替换为 Lottie / PixiJS 渲染器，无需改动状态机。
