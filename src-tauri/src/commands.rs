use crate::model::SystemStats;
use crate::state::{AppState, DockEdge};
use tauri::{Manager, PhysicalPosition, PhysicalSize, State};

/// 首屏立即取一次快照，避免等定时推送。
#[tauri::command]
pub fn get_stats(state: State<'_, AppState>) -> SystemStats {
    let mut monitor = state.monitor.lock().unwrap();
    monitor.sample()
}

/// 停靠态窗口尺寸（逻辑像素）：只留宠物探头的小方块，贴在工作区边缘。
const DOCK_W_LOGICAL: f64 = 118.0;
const DOCK_H_LOGICAL: f64 = 118.0;

fn dock_size(scale: f64) -> (u32, u32) {
    (
        (DOCK_W_LOGICAL * scale).round() as u32,
        (DOCK_H_LOGICAL * scale).round() as u32,
    )
}

/// 计算停靠/展开时窗口应处的物理坐标。
/// 两种状态都把窗口整体放进工作区、贴住对应边缘，只是尺寸不同。
fn edge_position(
    edge: DockEdge,
    work_x: i32,
    work_y: i32,
    work_w: u32,
    work_h: u32,
    win_w: i32,
    win_h: i32,
) -> (i32, i32) {
    match edge {
        DockEdge::Left => (work_x, work_y + (work_h as i32 - win_h) / 2),
        DockEdge::Right => (work_x + work_w as i32 - win_w, work_y + (work_h as i32 - win_h) / 2),
        DockEdge::Top => (work_x + (work_w as i32 - win_w) / 2, work_y),
    }
}

/// 把宠物停靠到屏幕边缘：左 / 右 / 上。
/// 停靠态把窗口缩成小方块贴在边缘（宠物探头可见），悬停再展开。
#[tauri::command]
pub fn dock(app: tauri::AppHandle, edge: String, state: State<'_, AppState>) -> Result<(), String> {
    let edge = match edge.as_str() {
        "left" => DockEdge::Left,
        "right" => DockEdge::Right,
        "top" => DockEdge::Top,
        other => return Err(format!("unknown dock edge: {other}")),
    };
    let win = app.get_webview_window("main").ok_or("main window not found")?;
    let monitor = win.current_monitor().ok().flatten().ok_or("no monitor found")?;
    let work = monitor.work_area();
    let scale = monitor.scale_factor();

    let pos = win.outer_position().map_err(|e| e.to_string())?;
    let size = win.inner_size().map_err(|e| e.to_string())?;

    let mut dock_state = state.dock.lock().unwrap();

    // 若已停靠，保留原始位置/尺寸；仅首次停靠时记录。
    // 若已是停靠态，用保存的原始尺寸作为展开基准，避免层层缩小。
    if dock_state.edge.is_none() {
        dock_state.saved_pos = Some((pos.x, pos.y));
        dock_state.saved_size = Some((size.width, size.height));
    }

    let (dw, dh) = dock_size(scale);
    let (x, y) = edge_position(
        edge,
        work.position.x,
        work.position.y,
        work.size.width,
        work.size.height,
        dw as i32,
        dh as i32,
    );

    win.set_size(PhysicalSize::new(dw, dh))
        .map_err(|e| e.to_string())?;
    win.set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;

    dock_state.edge = Some(edge);
    dock_state.expanded = false;
    Ok(())
}

/// 悬停展开：把停靠的小窗口恢复成完整尺寸，贴住边缘。
#[tauri::command]
pub fn expand_dock(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let dock_state = state.dock.lock().unwrap();
    let edge = match dock_state.edge {
        Some(e) => e,
        None => return Ok(()),
    };
    if dock_state.expanded {
        return Ok(());
    }
    let (saved_w, saved_h) = dock_state.saved_size.ok_or("no saved size")?;
    drop(dock_state);

    let win = app.get_webview_window("main").ok_or("main window not found")?;
    let monitor = win.current_monitor().ok().flatten().ok_or("no monitor found")?;
    let work = monitor.work_area();
    let (x, y) = edge_position(
        edge,
        work.position.x,
        work.position.y,
        work.size.width,
        work.size.height,
        saved_w as i32,
        saved_h as i32,
    );
    win.set_size(PhysicalSize::new(saved_w, saved_h))
        .map_err(|e| e.to_string())?;
    win.set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;

    let mut dock_state = state.dock.lock().unwrap();
    dock_state.expanded = true;
    Ok(())
}

/// 鼠标移出：把展开的窗口缩回小方块、贴住边缘。
#[tauri::command]
pub fn collapse_dock(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let dock_state = state.dock.lock().unwrap();
    let edge = match dock_state.edge {
        Some(e) => e,
        None => return Ok(()),
    };
    if !dock_state.expanded {
        return Ok(());
    }
    drop(dock_state);

    let win = app.get_webview_window("main").ok_or("main window not found")?;
    let monitor = win.current_monitor().ok().flatten().ok_or("no monitor found")?;
    let work = monitor.work_area();
    let scale = monitor.scale_factor();
    let (dw, dh) = dock_size(scale);
    let (x, y) = edge_position(
        edge,
        work.position.x,
        work.position.y,
        work.size.width,
        work.size.height,
        dw as i32,
        dh as i32,
    );
    win.set_size(PhysicalSize::new(dw, dh))
        .map_err(|e| e.to_string())?;
    win.set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;

    let mut dock_state = state.dock.lock().unwrap();
    dock_state.expanded = false;
    Ok(())
}

/// 从停靠态还原窗口到停靠前的位置与尺寸。
#[tauri::command]
pub fn undock(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let win = app.get_webview_window("main").ok_or("main window not found")?;
    let mut dock_state = state.dock.lock().unwrap();
    if let Some((w, h)) = dock_state.saved_size {
        win.set_size(PhysicalSize::new(w, h))
            .map_err(|e| e.to_string())?;
    }
    if let Some((x, y)) = dock_state.saved_pos {
        win.set_position(PhysicalPosition::new(x, y))
            .map_err(|e| e.to_string())?;
    }
    dock_state.edge = None;
    dock_state.expanded = false;
    dock_state.saved_pos = None;
    dock_state.saved_size = None;
    Ok(())
}

/// 查询当前停靠状态，返回边缘名或 null。
#[tauri::command]
pub fn is_docked(state: State<'_, AppState>) -> Option<String> {
    state.dock.lock().unwrap().edge.map(|e| match e {
        DockEdge::Left => "left".to_string(),
        DockEdge::Right => "right".to_string(),
        DockEdge::Top => "top".to_string(),
    })
}

/// 右键菜单「退出」：真正结束进程（关闭窗口只会隐藏，用于常驻）。
#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}
