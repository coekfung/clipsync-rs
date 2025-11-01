use tauri::{
    menu::{Menu, MenuEvent, MenuItem},
    tray::{MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, RunEvent, Runtime, WebviewWindowBuilder,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn init_tray_menu<R: Runtime>(app: &App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_i])?;

    let menu_event_handler = |app: &AppHandle<R>, event: MenuEvent| match event.id.as_ref() {
        "quit" => {
            app.exit(0);
        }
        _ => {
            eprintln!("No handler for menu item: {:?}", event.id);
        }
    };

    let tray_event_handler = |icon: &TrayIcon<R>, event: TrayIconEvent| -> anyhow::Result<()> {
        match event {
            TrayIconEvent::Click { button, .. } if button == MouseButton::Left => {
                let window = if let Some(window) = icon.app_handle().get_webview_window("main") {
                    window
                } else {
                    WebviewWindowBuilder::new(
                        icon.app_handle(),
                        "main",
                        tauri::WebviewUrl::App("index.html".into()),
                    )
                    .build()?
                };

                window.show()?;
                window.set_focus()?;
                Ok(())
            }
            _ => Ok(()),
        }
    };

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(menu_event_handler)
        .on_tray_icon_event(move |icon, event| {
            if let Err(e) = tray_event_handler(icon, event) {
                eprintln!("Tray icon event handler error: {:?}", e);
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| init_tray_menu(app))
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
