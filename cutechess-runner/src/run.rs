use color_print::cprintln;
use derive_getters::Getters;
use derive_new::new;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, exit};
use std::str::FromStr;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, Serialize, Deserialize, EnumIter, EnumString, Display)]
pub enum MatchSetup {
    #[strum(serialize = "Compliance")]
    Compliance,
    #[strum(serialize = "Bot v Bot")]
    BotVBot,
    #[strum(serialize = "Bot v White Human")]
    BotVWhiteHuman,
    #[strum(serialize = "Bot v Black Human")]
    BotVBlackHuman,
    #[strum(serialize = "Human v Human")]
    HumanVHuman,
}

#[derive(new, Getters)]
pub struct ChessOptions {
    setup: MatchSetup,
}

fn bot_v_bot(engine_path: PathBuf) {
    let mut command = Command::new("fastchess");
    let command = command
        .arg("-engine")
        .arg(format!("cmd={}", engine_path.display()))
        .arg("name=E1")
        .arg("-engine")
        .arg(format!("cmd={}", engine_path.display()))
        .arg("name=E2")
        .arg("-each")
        .arg("tc=5+0")
        .arg("proto=uci")
        .arg(format!(
            "dir=\"{}\"",
            PathBuf::from_str(".")
                .unwrap()
                .canonicalize()
                .unwrap()
                .display()
        ))
        .arg("-games")
        .arg("1")
        .arg("-pgnout")
        .arg("png.png");

    cprintln!(
        "<c>Args: {}</>",
        command
            .get_args()
            .map(|s| s.to_string_lossy())
            .collect_vec()
            .join(" ")
    );

    let Ok(status) = command.status() else {
        cprintln!("<r,bold>Failed to run fastchess</>");
        exit(-1);
    };

    if status.success() {
        cprintln!("<g,bold>Match finished successfully!</>");
    } else {
        cprintln!("<r,bold>fastchess exited with code: {:?}</>", status.code());
        exit(-1);
    }
}

fn compliance(engine_path: PathBuf) {
    let mut command = Command::new("fastchess");
    let command = command
        .arg("--compliance")
        .arg(engine_path.display().to_string());

    cprintln!(
        "<c>Args: {}</>",
        command
            .get_args()
            .map(|s| s.to_string_lossy())
            .collect_vec()
            .join(" ")
    );

    let Ok(status) = command.status() else {
        cprintln!("<r,bold>Failed to run fastchess</>");
        exit(-1);
    };

    if status.success() {
        cprintln!("<g,bold>Tool ran successfully!</>");
    } else {
        cprintln!("<r,bold>fastchess exited with code: {:?}</>", status.code());
        exit(-1);
    }
}

pub fn run(options: ChessOptions, engine_path: PathBuf) {
    match options.setup() {
        MatchSetup::Compliance => compliance(engine_path),
        _ => todo!(),
    }
}
