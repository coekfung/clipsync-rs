use clipsync_ipc::Commands;
use tauri::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, RunEvent, Runtime, WebviewWindow, WebviewWindowBuilder,
};

struct CommandsImpl;

#[clipsync_macros::command]
impl Commands for CommandsImpl {
    async fn hello(name: String) -> String {
        format!("Hello, {}! You've been greeted from Rust!", name)
    }
}

fn build_main_window_invisible<R: Runtime>(
    handle: &AppHandle<R>,
) -> tauri::Result<WebviewWindow<R>> {
    Ok(
        WebviewWindowBuilder::new(handle, "main", tauri::WebviewUrl::App("index.html".into()))
            .title("ClipSync")
            .visible(false) // show after restoring state
            .build()?,
    )
}

fn open_window_or_focus<R: Runtime>(app: &AppHandle<R>) -> anyhow::Result<()> {
    let window = if let Some(window) = app.get_webview_window("main") {
        window
    } else {
        build_main_window_invisible(app)?
    };

    window.show()?;
    window.set_focus()?;
    Ok(())
}

fn init_tray_menu<R: Runtime>(app: &App<R>) -> anyhow::Result<()> {
    let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
    let restart_i = MenuItem::with_id(app, "restart", "Restart", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &open_i,
            &PredefinedMenuItem::separator(app)?,
            &restart_i,
            &quit_i,
        ],
    )?;

    let menu_event_handler = |app: &AppHandle<R>, event: MenuEvent| match event.id.as_ref() {
        "open" => {
            if let Err(e) = open_window_or_focus(app) {
                eprintln!("Failed to open or focus window: {:?}", e);
            }
        }
        "restart" => {
            app.restart();
        }
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
                open_window_or_focus(icon.app_handle())
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
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            init_tray_menu(app)?;

            build_main_window_invisible(app.handle())?.show()?;

            Ok(())
        })
        // .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(CommandsImpl::invoke_handler)
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
