use crate::monitor::Monitor;

/// 全局共享状态：采集器被首屏命令与后台定时器共用，需串行访问。
pub struct AppState {
    pub monitor: std::sync::Mutex<Monitor>,
    pub interval_ms: std::sync::atomic::AtomicU64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            monitor: std::sync::Mutex::new(Monitor::new()),
            interval_ms: std::sync::atomic::AtomicU64::new(1500),
        }
    }
}
