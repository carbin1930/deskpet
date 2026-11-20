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
            let mut monitor = app.state::<AppState>().monitor.lock().unwrap();
            monitor.sample()
        };

        let _ = app.emit("stats://tick", snapshot);
        std::thread::sleep(std::time::Duration::from_millis(interval));
    });
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::new(app, "show", "显示/隐藏", true, None::<&str>)?;
    let quit = MenuItem::new(app, "quit", "退出", true, None::<&str>)?;
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
        .invoke_handler(tauri::generate_handler![commands::get_stats])
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
        })
        .run(tauri::generate_context!())
        .expect("error while running deskpet");
}
