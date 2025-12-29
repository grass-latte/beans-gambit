pub mod bot_state;
pub mod commands;
mod debug;
mod internal;

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
