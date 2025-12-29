use color_print::cprintln;
use derive_getters::Getters;
use derive_new::new;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, exit};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, Serialize, Deserialize, EnumIter, EnumString, Display)]
pub enum MatchSetup {
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

pub fn run(options: ChessOptions, engine_path: PathBuf) {
    let mut command = Command::new("cutechess-cli");
    let command = command
        .arg("-engine")
        .arg(format!("cmd=\"{}\"", engine_path.display()))
        .arg("-engine")
        .arg(format!("cmd=\"{}\"", engine_path.display()))
        .arg("-each")
        .arg("tc=5+0")
        .arg("-games")
        .arg("1")
        .arg("-pgnout")
        .arg("png.png");

    cprintln!(
        "<c>Args: {}</>",
        command
            .get_args()
            .into_iter()
            .map(|s| s.to_string_lossy())
            .collect_vec()
            .join(" ")
    );

    let Ok(status) = command.status() else {
        cprintln!("<r,bold>Failed to run cutechess-cli</>");
        exit(-1);
    };

    if status.success() {
        cprintln!("<g,bold>Match finished successfully!</>");
    } else {
        cprintln!(
            "<r,bold>cutechess-cli exited with code: {:?}</>",
            status.code()
        );
        exit(-1);
    }
}
