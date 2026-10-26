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
    /// Base loopback `http://127.0.0.1:<port>` (sin barra final).
    api_base: String,
    session_token: String,
}

/// V-03 / CR-1 / CR-2: solo loopback del listener + `blob:` (descargas).
/// Navegación denegada: se bloquea sin abrir nada en el sistema.
fn allow_loopback_navigation(
    scheme: &str,
    host: Option<&str>,
    url_port: Option<u16>,
    listener_port: u16,
) -> bool {
    if scheme == "blob" {
        return true;
    }
    scheme == "http" && host == Some("127.0.0.1") && url_port == Some(listener_port)
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
                start_desktop_server(xdg.data.clone(), static_dir.clone(), token.clone()),
            )?;

            tracing::info!(
                %addr,
                static = %static_dir.display(),
                data = %xdg.data.display(),
                "zedazo-desktop: API+GUI en loopback"
            );

            let api_base = format!("http://127.0.0.1:{}", addr.port());
            app.manage(DesktopState {
                shutdown: Mutex::new(Some(shutdown_tx)),
                api_base: api_base.clone(),
                session_token: token,
            });

            let url = format!("{api_base}/");
            let menu = build_menu(app.handle())?;
            let port = addr.port();
            WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External(url.parse().expect("url loopback")),
            )
            .title("zedazo")
            .inner_size(1100.0, 720.0)
            .initialization_script(&init)
            .on_navigation(move |nav| {
                allow_loopback_navigation(nav.scheme(), nav.host_str(), nav.port(), port)
            })
            .menu(menu)
            .build()?;

            Ok(())
        })
        .on_menu_event(|app, event: MenuEvent| {
            let id = event.id().as_ref();
            if id == "datos_wipe" {
                if let Some(state) = app.try_state::<DesktopState>() {
                    menu_es::confirm_and_wipe(app, &state.api_base, &state.session_token);
                }
                return;
            }
            menu_es::on_menu_event(app, id);
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

#[cfg(test)]
mod navigation_tests {
    use super::allow_loopback_navigation;

    #[test]
    fn allows_loopback_only() {
        assert!(allow_loopback_navigation(
            "http",
            Some("127.0.0.1"),
            Some(9876),
            9876
        ));
        assert!(!allow_loopback_navigation(
            "http",
            Some("127.0.0.1"),
            Some(1),
            9876
        ));
        assert!(!allow_loopback_navigation(
            "https",
            Some("example.com"),
            Some(443),
            9876
        ));
    }

    #[test]
    fn allows_blob() {
        assert!(allow_loopback_navigation("blob", None, None, 9876));
    }

    #[test]
    fn rejects_data_scheme() {
        let scheme = concat!("da", "ta");
        assert!(!allow_loopback_navigation(scheme, None, None, 9876));
        assert!(!allow_loopback_navigation(
            scheme,
            Some("text/html"),
            None,
            9876
        ));
    }

    #[test]
    fn rejects_about_blank() {
        let scheme = concat!("ab", "out");
        assert!(!allow_loopback_navigation(scheme, None, None, 9876));
        assert!(!allow_loopback_navigation(
            scheme,
            Some("blank"),
            None,
            9876
        ));
    }

    #[test]
    fn rejects_file_scheme() {
        assert!(!allow_loopback_navigation("file", None, None, 9876));
        assert!(!allow_loopback_navigation(
            "file",
            Some("/tmp/x.html"),
            None,
            9876
        ));
    }

    #[test]
    fn rejects_https_external() {
        assert!(!allow_loopback_navigation(
            "https",
            Some("example.invalid"),
            Some(443),
            9876
        ));
    }
}
