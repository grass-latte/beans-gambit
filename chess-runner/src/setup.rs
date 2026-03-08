use color_print::cprintln;
use derive_getters::Getters;
use derive_new::new;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use either::Either;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str::FromStr;
use std::sync::LazyLock;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount, EnumIter, EnumString};
use chess_lib::board::Board;

pub struct BotVsBotOptions {
    pub games: usize,
}

impl Default for BotVsBotOptions {
    fn default() -> Self {
        BotVsBotOptions { games: 5 }
    }
}

pub struct PerformanceOptions {
    pub fen: String,
}

impl Default for PerformanceOptions {
    fn default() -> Self {
        PerformanceOptions {
            fen: String::from("r2q1rk1/ppp2ppp/2npbn2/3Np3/2P1P3/2N1BP2/PP3P1P/R2QKB1R w KQ - 0 10"),
        }
    }
}


pub enum MatchType {
    BotVsBot(BotVsBotOptions),
    Compliance,
    Performance(PerformanceOptions),
    BuildOnly,
    BuildAndRun,
}

impl MatchType {
    pub fn setup_bot_vs_bot() -> MatchType {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Configure Bot v Bot")
            .items(vec!["Default", "Custom"])
            .default(0)
            .interact()
            .unwrap();

        if selection == 0 {
            MatchType::BotVsBot(BotVsBotOptions::default())
        } else {
            let i = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter number of games (1-1000)")
                .validate_with(|i: &String| {
                    if i.parse::<usize>().is_ok_and(|i| (1..=1000).contains(&i)) {
                        Ok(())
                    } else {
                        Err("Invalid number of games")
                    }
                })
                .interact()
                .unwrap();

            MatchType::BotVsBot(BotVsBotOptions {
                games: i.parse::<usize>().unwrap(),
            })
        }
    }

    pub fn setup_performance() -> MatchType {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Configure Performance Test")
            .items(vec!["Default Game", "Custom Fen"])
            .default(0)
            .interact()
            .unwrap();

        if selection == 0 {
            MatchType::Performance(PerformanceOptions::default())
        } else {
            let i = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter FEN: ")
                .validate_with(|i: &String| {
                    Board::from_fen(i).map(|_| ())
                })
                .interact()
                .unwrap();

            MatchType::Performance(PerformanceOptions {
                fen: i,
            })
        }
    }
}

#[derive(Debug, Serialize, Deserialize, EnumIter, EnumString, Display)]
pub enum SimpleMatchType {
    #[strum(serialize = "Bot v Bot")]
    BotVsBot,
    #[strum(serialize = "Compliance")]
    Compliance,
    #[strum(serialize = "Performance (Flamegraph)")]
    Performance,
    #[strum(serialize = "Build Only")]
    BuildOnly,
    #[strum(serialize = "Build and Run")]
    BuildAndRun,
}

impl SimpleMatchType {
    pub fn bots_required(&self) -> usize {
        match &self {
            SimpleMatchType::BotVsBot => 2,
            SimpleMatchType::Compliance => 1,
            SimpleMatchType::Performance => 1,
            SimpleMatchType::BuildOnly => 1,
            SimpleMatchType::BuildAndRun => 1,
        }
    }

    pub fn complete_setup(&self) -> MatchType {
        match &self {
            SimpleMatchType::BotVsBot => MatchType::setup_bot_vs_bot(),
            SimpleMatchType::Compliance => MatchType::Compliance,
            SimpleMatchType::Performance => MatchType::setup_performance(),
            SimpleMatchType::BuildOnly => MatchType::BuildOnly,
            SimpleMatchType::BuildAndRun => MatchType::BuildAndRun,
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
        cprintln!("<y,bold>Failed to execute git command to fetch remote versions</>");
        return vec![];
    };

    if !output.status.success() {
        cprintln!("<y,bold>git command to fetch remote versions returned failure exit code</>");
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

#[derive(Debug, Serialize, Deserialize, EnumString, EnumIter, Display, EnumCount, Hash)]
pub enum LocalBot {
    #[strum(serialize = "Beans Gambit [local]")]
    BeansGambitLocal,
    #[strum(serialize = "Stockfish")]
    Stockfish,
}

impl LocalBot {
    pub fn get_available() -> Vec<String> {
        let mut options = vec![LocalBot::BeansGambitLocal.to_string()];

        if which::which("stockfish").is_ok() {
            options.push(LocalBot::Stockfish.to_string());
        } else {
            cprintln!("<yellow,bold>Stockfish not found</>");
        }

        options
    }
}

fn select_bot(index: usize) -> Either<LocalBot, String> {
    let mut versions = LocalBot::get_available();

    let available_remote_versions: &Vec<String> = AVAILABLE_VERSIONS.as_ref();

    versions.extend(
        available_remote_versions
            .iter()
            .map(|s| format!("Beans Gambit [{s}]")),
    );

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Select bot {index}"))
        .items(&versions)
        .default(0)
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
    let items: Vec<_> = SimpleMatchType::iter().map(|a| a.to_string()).collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select option")
        .items(&items)
        .interact()
        .unwrap();
    let selection = SimpleMatchType::from_str(&items[selection]).unwrap();

    let mut bots = Vec::new();

    for i in 0..selection.bots_required() {
        bots.push(select_bot(i));
    }

    (ChessOptions::new(selection.complete_setup()), bots)
}


#[cfg(test)]
mod tests {
    use super::*;

    fn check_valid_default_fen() {
        Board::from_fen(&PerformanceOptions::default().fen).unwrap();
    }
}