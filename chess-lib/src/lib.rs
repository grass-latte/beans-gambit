#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
pub mod board;
pub mod movegen;
pub mod util;

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
