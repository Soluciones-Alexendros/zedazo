//! Biblioteca del escritorio: loopback, sesión, XDG (testeable sin WebKit).

pub mod loopback;
pub mod menu_es;
pub mod session;
pub mod xdg;

pub use loopback::{
    desktop_router, resolve_static_dir, resolve_static_dir_from, start_desktop_server,
    LoopbackGuard, StaticDirError,
};
pub use session::{generate_session_token, init_script};
pub use xdg::XdgPaths;
