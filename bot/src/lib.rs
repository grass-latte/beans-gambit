pub mod bot_state;
pub mod commands;
mod debug;
mod internal;

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub const fn chess_lib_version() -> &'static str {
    chess_lib::version()
}
