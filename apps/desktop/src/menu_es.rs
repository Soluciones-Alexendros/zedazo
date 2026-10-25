//! Menú nativo en español (Ayuda con guía embebida mínima).

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager};

const HELP_EVENT: &str = "zedazo://ayuda";

/// Texto mínimo embebido (W5B ampliará docs/desktop.md).
pub const GUIA_RAPIDA: &str = "\
zedazo — escritorio local (AppImage)\n\
\n\
• La API y la GUI corren en el mismo proceso en 127.0.0.1 (puerto efímero).\n\
• No hay red saliente por defecto; los datos quedan bajo XDG (config/data/state).\n\
• Autenticación: token de sesión inyectado al webview (Bearer); no reutiliza el login remoto.\n\
• Si el AppImage no arranca: pruebe --appimage-extract-and-run (FUSE) o revise WebKitGTK.\n\
• Guía completa: docs/desktop.md (oleada W5B).\n";

pub fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let ayuda_item = MenuItem::with_id(app, "ayuda_guia", "Guía rápida", true, None::<&str>)?;
    let acerca = MenuItem::with_id(app, "ayuda_acerca", "Acerca de zedazo", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let submenu = Submenu::with_items(app, "Ayuda", true, &[&ayuda_item, &sep, &acerca])?;
    Menu::with_items(app, &[&submenu])
}

pub fn on_menu_event(app: &AppHandle, id: &str) {
    match id {
        "ayuda_guia" => {
            let _ = app.emit(HELP_EVENT, GUIA_RAPIDA);
            // Diálogo nativo vía mensaje en stderr / log; ventana web puede escuchar el evento.
            tracing::info!(target: "zedazo_desktop", "{GUIA_RAPIDA}");
            if let Some(win) = app.get_webview_window("main") {
                let js = format!(
                    "window.alert({});",
                    serde_json::to_string(GUIA_RAPIDA).unwrap_or_else(|_| "\"Ayuda\"".into())
                );
                let _ = win.eval(&js);
            }
        }
        "ayuda_acerca" => {
            let msg = format!(
                "zedazo {}\nEscritorio AppImage (ADR-0021)\nAPI same-process en loopback.",
                env!("CARGO_PKG_VERSION")
            );
            if let Some(win) = app.get_webview_window("main") {
                let js = format!(
                    "window.alert({});",
                    serde_json::to_string(&msg).unwrap_or_else(|_| "\"zedazo\"".into())
                );
                let _ = win.eval(&js);
            }
        }
        _ => {}
    }
}
