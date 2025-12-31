use color_print::{cformat, cprint, cprintln};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

fn cutechess_in_path() -> bool {
    Command::new("fastchess").arg("--version").output().is_ok()
}

pub fn prerequisites() -> PathBuf {
    if cfg!(debug_assertions) {
        cprintln!("<cyan>Running in debug mode</>");
    } else {
        cprintln!("<cyan,bold>Running in <w>release</> mode</>");
    }

    cprint!("<yellow,bold>Testing fastchess binary...</>");
    if !cutechess_in_path() {
        panic!(
            "{}",
            cformat!("<r,bold>fastchess binary not found</>         ")
        );
    }
    cprintln!("\r<green>fastchess binary found</>         ");

    cprint!("<yellow,bold>Getting build location...</>");

    let Ok(output) = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output()
    else {
        panic!("{}", cformat!("<r,bold>Failed to run cargo metadata</>"));
    };
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) else {
        panic!(
            "{}",
            cformat!("<r,bold>Failed to extract json data from cargo metadata</>")
        );
    };
    let Some(target_dir) = json["target_directory"].as_str() else {
        panic!(
            "{}",
            cformat!("<r,bold>Failed to extract target_directory from cargo metadata</>")
        );
    };

    let Ok(target_dir) = PathBuf::from_str(target_dir);

    cprintln!(
        "\r<cyan>Target dir: <w>{}</></>         ",
        target_dir.display()
    );

    let command: &[&str] = if cfg!(debug_assertions) {
        &[
            "cargo",
            "build",
            "--package",
            "engine-uci",
            "--bin",
            "engine-uci",
        ]
    } else {
        &[
            "cargo",
            "build",
            "--package",
            "engine-uci",
            "--bin",
            "engine-uci",
            "--release",
        ]
    };

    cprintln!(
        "<yellow,bold>Building engine-uci from package engine-uci <w>[{}]</>...</>",
        command.join(" ")
    );

    let Ok(status) = Command::new(command[0])
        .args(&command[1..])
        .env("RUSTFLAGS", "-Awarnings")
        .status()
    else {
        panic!("{}", cformat!("<r,bold>Failed to run cargo</>"));
    };

    if !status.success() {
        panic!("{}", cformat!("<r,bold>Build command failed</>"));
    }

    let exe_path = if cfg!(debug_assertions) {
        target_dir.join("debug").join("engine-uci")
    } else {
        target_dir.join("release").join("engine-uci")
    };
    if !exe_path.is_file() {
        panic!(
            "{}",
            cformat!("<r,bold>Executable not found at {}</>", exe_path.display())
        );
    }

    cprintln!("<c>Found executable at {}</>", exe_path.display());

    exe_path
}
