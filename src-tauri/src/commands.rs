use crate::model::SystemStats;
use crate::state::{AppState, DockEdge};
use tauri::{Manager, PhysicalPosition, State};

/// 首屏立即取一次快照，避免等定时推送。
#[tauri::command]
pub fn get_stats(state: State<'_, AppState>) -> SystemStats {
    let mut monitor = state.monitor.lock().unwrap();
    monitor.sample()
}

/// 停靠后露出的边缘宽度（逻辑像素），鼠标悬停时展开。
const PEEK_LOGICAL: f64 = 54.0;

/// 计算停靠态下窗口应处的物理坐标。
/// 停靠 = 窗口主体移出工作区，只在对应边缘露出 PEEK 宽度。
fn docked_position(
    edge: DockEdge,
    win_x: i32,
    win_y: i32,
    win_w: u32,
    win_h: u32,
    work_x: i32,
    work_y: i32,
    work_w: u32,
    _work_h: u32,
    peek: i32,
) -> (i32, i32) {
    match edge {
        DockEdge::Left => (work_x - (win_w as i32 - peek), win_y),
        DockEdge::Right => (work_x + work_w as i32 - peek, win_y),
        DockEdge::Top => (win_x, work_y - (win_h as i32 - peek)),
    }
}

/// 计算悬停展开时窗口应处的物理坐标（整窗滑入工作区）。
fn expanded_position(
    edge: DockEdge,
    win_x: i32,
    win_y: i32,
    win_w: u32,
    _win_h: u32,
    work_x: i32,
    work_y: i32,
    work_w: u32,
    _work_h: u32,
) -> (i32, i32) {
    match edge {
        DockEdge::Left => (work_x, win_y),
        DockEdge::Right => (work_x + work_w as i32 - win_w as i32, win_y),
        DockEdge::Top => (win_x, work_y),
    }
}

/// 把宠物停靠到屏幕边缘：左 / 右 / 上。
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
    let peek = (PEEK_LOGICAL * scale).round() as i32;

    let pos = win.outer_position().map_err(|e| e.to_string())?;
    let size = win.inner_size().map_err(|e| e.to_string())?;

    let mut dock_state = state.dock.lock().unwrap();

    // 若已停靠， retains 原始位置/尺寸；仅首次停靠时记录。
    if dock_state.edge.is_none() {
        dock_state.saved_pos = Some((pos.x, pos.y));
        dock_state.saved_size = Some((size.width, size.height));
    }

    // 用当前（或还原后）的尺寸计算停靠坐标。
    let (cur_x, cur_y) = dock_state.saved_pos.unwrap_or((pos.x, pos.y));
    let (cur_w, cur_h) = dock_state.saved_size.unwrap_or((size.width, size.height));

    let (x, y) = docked_position(
        edge,
        cur_x,
        cur_y,
        cur_w,
        cur_h,
        work.position.x,
        work.position.y,
        work.size.width,
        work.size.height,
        peek,
    );

    win.set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;

    dock_state.edge = Some(edge);
    dock_state.expanded = false;
    Ok(())
}

/// 悬停展开：把停靠的窗口整窗滑入工作区。
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
    let (saved_x, saved_y) = dock_state.saved_pos.ok_or("no saved position")?;
    let (saved_w, saved_h) = dock_state.saved_size.ok_or("no saved size")?;
    drop(dock_state);

    let win = app.get_webview_window("main").ok_or("main window not found")?;
    let monitor = win.current_monitor().ok().flatten().ok_or("no monitor found")?;
    let work = monitor.work_area();
    let (x, y) = expanded_position(
        edge,
        saved_x,
        saved_y,
        saved_w,
        saved_h,
        work.position.x,
        work.position.y,
        work.size.width,
        work.size.height,
    );
    win.set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;

    let mut dock_state = state.dock.lock().unwrap();
    dock_state.expanded = true;
    Ok(())
}

/// 鼠标移出：把展开的窗口缩回边缘。
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
    let (saved_x, saved_y) = dock_state.saved_pos.ok_or("no saved position")?;
    let (saved_w, saved_h) = dock_state.saved_size.ok_or("no saved size")?;
    drop(dock_state);

    let win = app.get_webview_window("main").ok_or("main window not found")?;
    let monitor = win.current_monitor().ok().flatten().ok_or("no monitor found")?;
    let work = monitor.work_area();
    let scale = monitor.scale_factor();
    let peek = (PEEK_LOGICAL * scale).round() as i32;
    let (x, y) = docked_position(
        edge,
        saved_x,
        saved_y,
        saved_w,
        saved_h,
        work.position.x,
        work.position.y,
        work.size.width,
        work.size.height,
        peek,
    );
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
