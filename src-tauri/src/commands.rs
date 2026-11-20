use crate::model::SystemStats;
use crate::state::AppState;
use tauri::State;

/// 首屏立即取一次快照，避免等定时推送。
#[tauri::command]
pub fn get_stats(state: State<'_, AppState>) -> SystemStats {
    let mut monitor = state.monitor.lock().unwrap();
    monitor.sample()
}
