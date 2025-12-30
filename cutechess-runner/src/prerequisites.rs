use color_print::{cprint, cprintln};
use std::path::PathBuf;
use std::process::{Command, exit};
use std::str::FromStr;

fn cutechess_in_path() -> bool {
    Command::new("cutechess-cli")
        .arg("--version")
        .output()
        .is_ok()
}

pub fn prerequisites() -> PathBuf {
    if cfg!(debug_assertions) {
        cprintln!("<cyan>Running in debug mode</>");
    } else {
        cprintln!("<cyan,bold>Running in <w>release</> mode</>");
    }

    cprint!("<yellow,bold>Testing cutechess-cli binary...</>");
    if !cutechess_in_path() {
        cprintln!("\r<r,bold>cutechess-cli binary not found</>         ");
        exit(-1);
    }
    cprintln!("\r<green>cutechess-cli binary found</>         ");

    cprint!("<yellow,bold>Getting build location...</>");

    let Ok(output) = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output()
    else {
        cprintln!("\r<r,bold>Failed to run cargo metadata</>         ");
        exit(-1);
    };
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) else {
        cprintln!("\r<r,bold>Failed to extract json data from cargo metadata</>         ");
        exit(-1);
    };
    let Some(target_dir) = json["target_directory"].as_str() else {
        cprintln!("\r<r,bold>Failed to extract target_directory from cargo metadata</>         ");
        exit(-1);
    };

    let Ok(target_dir) = PathBuf::from_str(target_dir) else {
        cprintln!("\r<r,bold>Failed to convert target_directory to PathBuf</>         ");
        exit(-1);
    };

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

    let Ok(status) = Command::new(&command[0])
        .args(&command[1..])
        .env("RUSTFLAGS", "-Awarnings")
        .status()
    else {
        cprintln!("<r,bold>Failed to run cargo</>");
        exit(-1);
    };

    if !status.success() {
        cprintln!("<r,bold>Build command failed</>");
        exit(-1);
    }

    let exe_path = if cfg!(debug_assertions) {
        target_dir.join("debug").join("engine-uci")
    } else {
        target_dir.join("release").join("engine-uci")
    };
    if !exe_path.is_file() {
        cprintln!("<r,bold>Executable not found at {}</>", exe_path.display());
        exit(-1);
    }

    cprintln!("<c>Found executable at {}</>", exe_path.display());

    exe_path
}
