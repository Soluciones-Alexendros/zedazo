//! Binario Tauri: API same-process en 127.0.0.1:0 + webview (ADR-0021).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use tauri::menu::MenuEvent;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tracing_subscriber::EnvFilter;
use zedazo_desktop::loopback::{resolve_static_dir, start_desktop_server};
use zedazo_desktop::menu_es::{self, build_menu};
use zedazo_desktop::session::{generate_session_token, init_script};
use zedazo_desktop::xdg::XdgPaths;

struct DesktopState {
    shutdown: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("zedazo_desktop=info".parse().unwrap()),
        )
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let xdg = XdgPaths::resolve()?;
            let static_dir = resolve_static_dir();
            let token = generate_session_token()?;
            let init = init_script(&token)?;

            let (addr, shutdown_tx, _server) = tauri::async_runtime::block_on(
                start_desktop_server(xdg.data.clone(), static_dir.clone(), token),
            )?;

            tracing::info!(
                %addr,
                static = %static_dir.display(),
                data = %xdg.data.display(),
                "zedazo-desktop: API+GUI en loopback"
            );

            app.manage(DesktopState {
                shutdown: Mutex::new(Some(shutdown_tx)),
            });

            let url = format!("http://127.0.0.1:{}/", addr.port());
            let menu = build_menu(app.handle())?;
            WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External(url.parse().expect("url loopback")),
            )
            .title("zedazo")
            .inner_size(1100.0, 720.0)
            .initialization_script(&init)
            .menu(menu)
            .build()?;

            Ok(())
        })
        .on_menu_event(|app, event: MenuEvent| {
            menu_es::on_menu_event(app, event.id().as_ref());
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(state) = window.try_state::<DesktopState>() {
                    if let Ok(mut guard) = state.shutdown.lock() {
                        if let Some(tx) = guard.take() {
                            let _ = tx.send(());
                        }
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error al ejecutar zedazo-desktop");
}
