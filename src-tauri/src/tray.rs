use crate::store::{effective_language, Settings};
use crate::AppState;
use std::sync::atomic::Ordering;
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime};

pub const TRAY_ID: &str = "model-desk-tray";

struct Labels {
    toggle: &'static str,
    new_chat: &'static str,
    quit: &'static str,
    tooltip: &'static str,
}

fn labels(lang: &str) -> Labels {
    if lang == "fr" {
        Labels {
            toggle: "Afficher / Masquer",
            new_chat: "Nouvelle conversation",
            quit: "Quitter",
            tooltip: "Model Desk",
        }
    } else {
        Labels {
            toggle: "Show / Hide",
            new_chat: "New chat",
            quit: "Quit",
            tooltip: "Model Desk",
        }
    }
}

fn build_menu<R: Runtime>(app: &AppHandle<R>, lang: &str) -> tauri::Result<Menu<R>> {
    let labels = labels(lang);
    let toggle = MenuItemBuilder::with_id("toggle", labels.toggle).build(app)?;
    let new_chat = MenuItemBuilder::with_id("new_chat", labels.new_chat).build(app)?;
    let quit = MenuItemBuilder::with_id("quit", labels.quit).build(app)?;
    MenuBuilder::new(app)
        .items(&[&toggle, &new_chat, &quit])
        .build()
}

pub fn build<R: Runtime>(app: &AppHandle<R>, settings: &Settings) -> tauri::Result<()> {
    let lang = effective_language(settings);
    let labels = labels(lang);
    let menu = build_menu(app, lang)?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(labels.tooltip)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => toggle_window(app),
            "new_chat" => {
                show_window(app);
                let _ = app.emit("tray-new-chat", ());
            }
            "quit" => {
                app.state::<AppState>()
                    .quitting
                    .store(true, Ordering::SeqCst);
                app.exit(0);
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
                toggle_window(tray.app_handle());
            }
        });

    match tauri::image::Image::from_bytes(include_bytes!("../icons/64x64.png")) {
        Ok(icon) => {
            builder = builder.icon(icon);
        }
        Err(error) => {
            eprintln!("tray icon could not be loaded: {error}");
            if let Some(icon) = app.default_window_icon().cloned() {
                builder = builder.icon(icon);
            }
        }
    }

    builder.build(app)?;
    Ok(())
}

pub fn update_menu<R: Runtime>(app: &AppHandle<R>, settings: &Settings) {
    let lang = effective_language(settings);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        if let Ok(menu) = build_menu(app, lang) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn toggle_window<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    match window.is_visible() {
        Ok(true) => {
            let _ = window.hide();
        }
        _ => show_window(app),
    }
}

fn show_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
