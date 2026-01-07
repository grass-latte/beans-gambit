use crate::run::ChessBot;
use crate::setup::LocalBot;
use color_print::{cformat, cprint, cprintln};
use either::Either;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};

static VERSIONS_FETCHED: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static DIRECTORIES_BUILT: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

static SHOWN_COMPILE_MESSAGE: LazyLock<AtomicBool> = LazyLock::new(|| AtomicBool::new(false));

fn compile_directory(directory: PathBuf) -> PathBuf {
    if !SHOWN_COMPILE_MESSAGE.load(Ordering::SeqCst) {
        if cfg!(debug_assertions) {
            cprintln!("<cyan,bold>Bots will be built in <w>debug</> mode</>");
        } else {
            cprintln!("<cyan,bold>Bots will be built in <w>release</> mode</>");
        }
        SHOWN_COMPILE_MESSAGE.store(true, Ordering::SeqCst);
    }

    cprint!("<yellow,bold>Getting build location...</>");

    let Ok(output) = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .current_dir(&directory)
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

    if !DIRECTORIES_BUILT.lock().unwrap().contains(&target_dir) {
        let crate_to_build = fs::read_to_string(target_dir.join("uci-crate"))
            .unwrap_or("engine-uci".to_string())
            .trim()
            .to_string();

        let command: &[&str] = if cfg!(debug_assertions) {
            &[
                "cargo",
                "build",
                "--package",
                &crate_to_build,
                "--bin",
                &crate_to_build,
            ]
        } else {
            &[
                "cargo",
                "build",
                "--package",
                &crate_to_build,
                "--bin",
                &crate_to_build,
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
            .current_dir(&directory)
            .status()
        else {
            panic!("{}", cformat!("<r,bold>Failed to run cargo</>"));
        };

        if !status.success() {
            panic!("{}", cformat!("<r,bold>Build command failed</>"));
        }
        DIRECTORIES_BUILT.lock().unwrap().insert(target_dir.clone());
    } else {
        cprintln!("<green,bold>Already built!</>");
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

fn get_stockfish() -> ChessBot {
    let Ok(path) = which::which("stockfish") else {
        panic!(
            "{}",
            cformat!("<r,bold>Could not find stockfish executable.</>")
        );
    };

    ChessBot {
        name: "Stockfish".to_string(),
        path: path.display().to_string(),
    }
}

pub fn resolve_local_bot(local_bot: LocalBot) -> ChessBot {
    match local_bot {
        LocalBot::Stockfish => get_stockfish(),
        LocalBot::BeansGambitLocal => ChessBot {
            name: "Beans Gambit [Local]".to_string(),
            path: compile_directory(PathBuf::from_str(".").unwrap())
                .display()
                .to_string(),
        },
    }
}

pub fn resolve_remote_bot(version: String) -> ChessBot {
    if VERSIONS_FETCHED.lock().unwrap().contains(&version) {
        cprintln!("<green,bold>{version} already fetched!</>");
    } else {
        cprintln!("<c,bold>Fetching {version}...</>");

        if fs::create_dir_all("versions").is_err() {
            panic!(
                "{}",
                cformat!("<r,bold>Failed to create versions folder.</>")
            );
        }

        if fs::exists(format!("versions/{version}")).unwrap_or(false)
            && fs::remove_dir_all(format!("versions/{version}")).is_err()
        {
            panic!(
                "{}",
                cformat!("<r,bold>Failed to remove versions/{version}</>")
            );
        }

        let Ok(status) = Command::new("git")
            .arg("clone")
            .arg("-b")
            .arg(format!("versions/{version}"))
            .arg("https://github.com/grass-latte/beans-gambit.git")
            .arg(format!("versions/{version}"))
            .status()
        else {
            panic!("{}", cformat!("<r,bold>Failed to run git command</>"));
        };

        if !status.success() {
            panic!(
                "{}",
                cformat!("<r,bold>git clone of branch versions/{version} failed</>")
            );
        }

        VERSIONS_FETCHED.lock().unwrap().insert(version.clone());
    }

    ChessBot {
        name: format!("Beans Gambit [{version}]"),
        path: compile_directory(
            PathBuf::from_str(".")
                .unwrap()
                .join("versions")
                .join(version),
        )
        .display()
        .to_string(),
    }
}

pub fn resolve_bot(bot: Either<LocalBot, String>) -> ChessBot {
    match bot {
        Either::Left(local_bot) => resolve_local_bot(local_bot),
        Either::Right(version) => resolve_remote_bot(version),
    }
}
