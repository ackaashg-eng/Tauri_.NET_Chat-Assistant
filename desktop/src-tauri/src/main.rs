//! SPIL Labs Assistant — desktop app (Tauri 2).
//!
//! The app is a window onto the shared SPIL Labs server:
//!   1. On first start it shows a small "connect" screen (desktop/ui/index.html) that asks for the
//!      server address, checks it (GET /api/health) and remembers it.
//!   2. It then opens the server's own screens (chat, tools, admin) in this window, so every PC
//!      always gets the latest version without reinstalling.
//!
//! Menu  Server > Change server… (Ctrl+Shift+S) returns to the connect screen.
//! The server's pages get NO access to Tauri/desktop APIs (no "remote" capability is granted).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // no console window on Windows

use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::{Manager, Url};

/// Address of the bundled connect screen. Windows serves app files from http://tauri.localhost,
/// macOS/Linux from tauri://localhost.
fn connect_screen() -> Url {
    let base = if cfg!(windows) { "http://tauri.localhost/index.html" } else { "tauri://localhost/index.html" };
    Url::parse(&format!("{base}?change=1")).expect("valid url")
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let change = MenuItemBuilder::with_id("change-server", "Change server…")
                .accelerator("CmdOrCtrl+Shift+S")
                .build(app)?;
            let reload = MenuItemBuilder::with_id("reload", "Reload").accelerator("CmdOrCtrl+R").build(app)?;
            let quit = PredefinedMenuItem::quit(app, Some("Quit"))?;
            let server = SubmenuBuilder::new(app, "Server")
                .item(&change)
                .item(&reload)
                .separator()
                .item(&quit)
                .build()?;
            let menu = MenuBuilder::new(app).item(&server).build()?;
            app.set_menu(menu)?;

            app.on_menu_event(|app, event| {
                let Some(window) = app.get_webview_window("main") else { return };
                match event.id().as_ref() {
                    "change-server" => {
                        let _ = window.navigate(connect_screen());
                    }
                    "reload" => {
                        let _ = window.eval("window.location.reload()");
                    }
                    _ => {}
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the SPIL Labs desktop app");
}
