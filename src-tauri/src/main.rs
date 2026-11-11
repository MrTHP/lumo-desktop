#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod attachments;
mod install;
mod opencode;
mod store;
mod tools;
mod tray;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::Duration;
use store::Store;
use tauri::Manager;
use tokio_util::sync::CancellationToken;

pub struct AppState {
    pub store: Store,
    pub client: reqwest::Client,
    pub cancels: Mutex<HashMap<String, CancellationToken>>,
    pub permissions: Mutex<HashMap<String, tokio::sync::oneshot::Sender<String>>>,
    pub auto_approve: Mutex<HashSet<String>>,
    pub quitting: AtomicBool,
}

fn main() {
    let data_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lumo-desktop");

    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client");

    tauri::Builder::default()
        .manage(AppState {
            store: Store::new(data_dir),
            client,
            cancels: Mutex::new(HashMap::new()),
            permissions: Mutex::new(HashMap::new()),
            auto_approve: Mutex::new(HashSet::new()),
            quitting: AtomicBool::new(false),
        })
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128.png"))
                    .ok()
                    .or_else(|| app.default_window_icon().cloned());
                if let Some(icon) = icon {
                    let _ = window.set_icon(icon);
                }
            }

            let settings = app.state::<AppState>().store.load_settings();
            if let Err(error) = tray::build(app.handle(), &settings) {
                eprintln!("tray icon unavailable: {error}");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                if app.tray_by_id(tray::TRAY_ID).is_some()
                    && !app
                        .state::<AppState>()
                        .quitting
                        .load(std::sync::atomic::Ordering::SeqCst)
                {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            api::get_initial_state,
            api::get_dev_prompt,
            api::get_dev_agent,
            api::get_dev_open,
            api::get_dev_attachment,
            api::save_settings,
            api::save_conversations,
            api::check_health,
            api::list_models,
            api::detect_opencode_providers,
            api::extract_memories,
            api::start_chat,
            api::stop_chat,
            api::resolve_tool_permission,
            api::start_tamer,
            api::open_external,
            install::tamer_status,
            install::install_tamer,
            install::start_auth_browser,
            install::authenticate_tamer,
            install::get_web_search,
            install::set_web_search,
            opencode::opencode_status,
            opencode::list_opencode_models,
            opencode::start_agent,
            attachments::process_attachment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lumo Desktop");
}
