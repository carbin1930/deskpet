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
