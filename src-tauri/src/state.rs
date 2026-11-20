use crate::monitor::Monitor;
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

/// 窗口停靠到屏幕的哪条边。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockEdge {
    Left,
    Right,
    Top,
}

/// 停靠状态：记录停靠前的窗口位置与尺寸，用于还原。
#[derive(Default)]
pub struct DockState {
    pub edge: Option<DockEdge>,
    /// 停靠前的窗口左上角（物理像素）。
    pub saved_pos: Option<(i32, i32)>,
    /// 停靠前的窗口尺寸（物理像素）。
    pub saved_size: Option<(u32, u32)>,
    /// 当前是否处于「悬停展开」状态（停靠后鼠标移入时临时展开）。
    pub expanded: bool,
    /// 停靠目标屏幕的工作区（物理桌面坐标，x/y 可能为负）。
    /// 停靠时确定一次并缓存：展开/缩回若重新调 current_monitor()，
    /// 双屏下窗口贴边后中心点会落到相邻屏幕，导致位置错乱甚至移出屏幕外。
    pub work: Option<(i32, i32, u32, u32)>,
    /// 停靠目标屏幕的缩放因子。
    pub scale: f64,
}

/// 全局共享状态：采集器被首屏命令与后台定时器共用，需串行访问。
pub struct AppState {
    pub monitor: Mutex<Monitor>,
    pub interval_ms: AtomicU64,
    pub dock: Mutex<DockState>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            monitor: Mutex::new(Monitor::new()),
            interval_ms: AtomicU64::new(1500),
            dock: Mutex::new(DockState::default()),
        }
    }
}
