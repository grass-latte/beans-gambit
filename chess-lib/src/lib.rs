pub mod board;
pub mod movegen;
pub mod util;

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
