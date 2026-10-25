//! Biblioteca del escritorio: loopback, sesión, XDG (testeable sin WebKit).

pub mod loopback;
pub mod menu_es;
pub mod session;
pub mod xdg;

pub use loopback::{desktop_router, resolve_static_dir, start_desktop_server, LoopbackGuard};
pub use session::{generate_session_token, init_script};
pub use xdg::XdgPaths;
