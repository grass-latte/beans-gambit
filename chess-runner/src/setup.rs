use crate::run::{ChessOptions, MatchType};
use color_print::cprintln;
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use either::Either;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str::FromStr;
use std::sync::LazyLock;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount, EnumIter, EnumString};

fn get_remote_versions() -> Vec<String> {
    let Ok(output) = Command::new("git")
        .args([
            "ls-remote",
            "--heads",
            "https://github.com/grass-latte/beans-gambit.git",
            "refs/heads/versions/*",
        ])
        .output()
    else {
        cprintln!("<y,b>Failed to execute git command to fetch remote versions</>");
        return vec![];
    };

    if !output.status.success() {
        cprintln!("<y,b>git command to fetch remote versions returned failure exit code</>");
        return vec![];
    }

    let stdout = String::from_utf8(output.stdout).unwrap();

    let versions: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            line.split('\t')
                .nth(1)
                .and_then(|ref_name| ref_name.strip_prefix("refs/heads/versions/"))
                .map(|v| v.to_string())
        })
        .collect();

    versions
}

static AVAILABLE_VERSIONS: LazyLock<Vec<String>> = LazyLock::new(get_remote_versions);

#[derive(Debug, Serialize, Deserialize, EnumIter, EnumString, Display, EnumCount, Hash)]
pub enum LocalBot {
    #[strum(serialize = "Stockfish")]
    Stockfish,
    #[strum(serialize = "Beans Gambit [local]")]
    BeansGambitLocal,
}

fn select_bot(index: usize) -> Either<LocalBot, String> {
    let mut versions = LocalBot::iter().map(|n| n.to_string()).collect_vec();

    let available_remote_versions: &Vec<String> = AVAILABLE_VERSIONS.as_ref();

    versions.extend(
        available_remote_versions
            .iter()
            .map(|s| format!("Beans Gambit [{s}]")),
    );

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Select bot {index}"))
        .items(&versions)
        .interact()
        .unwrap();

    if selection < LocalBot::COUNT {
        Either::Left(LocalBot::from_str(&versions[selection]).unwrap())
    } else {
        let selection = selection - LocalBot::COUNT;
        Either::Right(AVAILABLE_VERSIONS[selection].clone())
    }
}

pub fn select_options() -> (ChessOptions, Vec<Either<LocalBot, String>>) {
    let items: Vec<_> = MatchType::iter().map(|a| a.to_string()).collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select option")
        .items(&items)
        .interact()
        .unwrap();
    let selection = MatchType::from_str(&items[selection]).unwrap();

    let mut bots = Vec::new();

    for i in 0..selection.bots_required() {
        bots.push(select_bot(i));
    }

    (ChessOptions::new(selection), bots)
}
