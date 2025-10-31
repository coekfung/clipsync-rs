use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, RunEvent,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let setting_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i, &setting_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("settings") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        } else {
                            let _settings_window = tauri::WebviewWindowBuilder::new(
                                app,
                                "settings",
                                tauri::WebviewUrl::App("index.html".into()),
                            )
                            .title("Settings")
                            .build()
                            .unwrap();
                        }
                    }
                    _ => todo!(),
                })
                .build(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            RunEvent::ExitRequested { code, api, .. } => {
                if code.is_none() {
                    // prevent app from exiting
                    api.prevent_exit();
                }
            }
            _ => {}
        });
}
