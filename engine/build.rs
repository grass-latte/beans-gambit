use chess_lib::board::Board;
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    eprintln!("Running engine build.rs");

    fs::write(
        "static/gen/hash.txt",
        format!("{}", Board::starting().hash()),
    )
    .unwrap();
}
