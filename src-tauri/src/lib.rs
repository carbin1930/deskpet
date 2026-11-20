mod commands;
mod model;
mod monitor;
mod state;

use state::AppState;
use tauri::{
    menu::{MenuBuilder, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

/// 启动后台采样线程，定时向前端推送 `stats://tick` 事件。
fn spawn_sampler(app: AppHandle) {
    std::thread::spawn(move || loop {
        let interval = app
            .state::<AppState>()
            .interval_ms
            .load(std::sync::atomic::Ordering::Relaxed);

        let snapshot = {
            let state = app.state::<AppState>();
            let mut monitor = state.monitor.lock().unwrap();
            monitor.sample()
        };

        let _ = app.emit("stats://tick", snapshot);
        std::thread::sleep(std::time::Duration::from_millis(interval));
    });
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示/隐藏", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

    let icon = app
        .default_window_icon()
        .cloned()
        .expect("default icon must be configured");

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = if win.is_visible().unwrap_or(false) {
                        win.hide()
                    } else {
                        win.show()
                    };
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_stats,
            commands::dock,
            commands::undock,
            commands::expand_dock,
            commands::collapse_dock,
            commands::is_docked,
            commands::quit_app
        ])
        .setup(|app| {
            build_tray(app.handle())?;
            spawn_sampler(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // 关闭按钮隐藏窗口而非退出进程，保持常驻
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
            // 停靠态下，若窗口位置明显偏离程序设定的位置，视为用户拖拽离开边缘：
            // 恢复完整尺寸并留在拖拽处。程序自身的 dock/expand/collapse 移动因位置
            // 与 expected_pos 一致，不会被误判。
            if let tauri::WindowEvent::Moved(pos) = event {
                let app = window.app_handle().clone();
                let detach = {
                    let state = app.state::<AppState>();
                    let ds = state.dock.lock().unwrap();
                    match ds.expected_pos {
                        Some((ex, ey)) if ds.edge.is_some() => {
                            (pos.x - ex).abs() > 16 || (pos.y - ey).abs() > 16
                        }
                        _ => false,
                    }
                };
                if detach {
                    let state = app.state::<AppState>();
                    commands::detach_by_drag(&app, &state);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running deskpet");
}
