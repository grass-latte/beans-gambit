use crate::setup::{BotVsBotOptions, ChessBot, ChessOptions, MatchType};
use color_print::{cformat, cprintln};
use itertools::Itertools;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

#[allow(unused)]
fn bot_vs_bot(bot1: ChessBot, bot2: ChessBot, options: &BotVsBotOptions) {
    let mut command = Command::new("fastchess");
    let command = command
        .arg("-engine")
        .arg(format!("cmd={}", &bot1.path))
        .arg(format!("name={}", &bot1.name))
        .arg("-engine")
        .arg(format!("cmd={}", &bot2.path))
        .arg(format!("name={}", &bot2.name))
        .arg("-each")
        .arg("tc=30+0")
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
        .arg(options.games.to_string())
        .arg("-pgnout")
        .arg("game.pgn");

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
        MatchType::BotVsBot(options) => bot_vs_bot(bots[0].clone(), bots[1].clone(), options),
        MatchType::BuildOnly => {
            println!("Bot {} at {}", &bots[0].name, &bots[0].path);
        }
        MatchType::BuildAndRun => {
            Command::new(&bots[0].path).status().unwrap();
        }
    }
}
