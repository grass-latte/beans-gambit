use color_print::{cformat, cprintln};
use derive_getters::Getters;
use derive_new::new;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, Serialize, Deserialize, EnumIter, EnumString, Display)]
pub enum MatchType {
    #[strum(serialize = "Bot v Bot")]
    BotVsBot,
    #[strum(serialize = "Compliance")]
    Compliance,
}

impl MatchType {
    pub fn bots_required(&self) -> usize {
        match &self {
            MatchType::BotVsBot => 2,
            MatchType::Compliance => 1,
        }
    }
}

#[derive(Clone)]
pub struct ChessBot {
    pub name: String,
    pub path: String,
}

#[derive(new, Getters)]
pub struct ChessOptions {
    setup: MatchType,
}

#[allow(unused)]
fn bot_vs_bot(bot1: ChessBot, bot2: ChessBot) {
    let mut command = Command::new("fastchess");
    let command = command
        .arg("-engine")
        .arg(format!("cmd={}", &bot1.path))
        .arg(format!("name={}", &bot1.name))
        .arg("-engine")
        .arg(format!("cmd={}", &bot2.path))
        .arg(format!("name={}", &bot2.name))
        .arg("-each")
        .arg("tc=5+0")
        .arg("proto=uci")
        .arg(format!(
            "dir={}",
            PathBuf::from_str(".")
                .unwrap()
                .canonicalize()
                .unwrap()
                .display()
        ))
        .arg("-games")
        .arg("1")
        .arg("-pgnout")
        .arg("game.txt");

    cprintln!(
        "<c>Args: {}</>",
        command
            .get_args()
            .map(|s| s.to_string_lossy())
            .collect_vec()
            .join(" ")
    );

    let Ok(status) = command.status() else {
        panic!("{}", cformat!("<r,bold>Failed to run fastchess</>"));
    };

    if status.success() {
        cprintln!("<g,bold>Match finished successfully!</>");
    } else {
        panic!(
            "{}",
            cformat!("<r,bold>fastchess exited with code: {:?}</>", status.code())
        );
    }
}

fn compliance(bot: ChessBot) {
    let mut command = Command::new("fastchess");
    let command = command.arg("--compliance").arg(bot.path);

    cprintln!(
        "<c>Args: {}</>",
        command
            .get_args()
            .map(|s| s.to_string_lossy())
            .collect_vec()
            .join(" ")
    );

    let Ok(status) = command.status() else {
        panic!("{}", cformat!("<r,bold>Failed to run fastchess</>"));
    };

    if status.success() {
        cprintln!("<g,bold>Tool ran successfully!</>");
    } else {
        panic!(
            "{}",
            cformat!("<r,bold>fastchess exited with code: {:?}</>", status.code())
        );
    }
}

pub fn run(options: ChessOptions, bots: Vec<ChessBot>) {
    match options.setup() {
        MatchType::Compliance => compliance(bots[0].clone()),
        MatchType::BotVsBot => bot_vs_bot(bots[0].clone(), bots[1].clone()),
    }
}
