use crate::bot_resolver::get_stockfish;
use crate::setup::ChessBot;
use color_print::{cprintln, cwrite};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

pub fn stockfish_ladder(bot: ChessBot) {
    let stockfish = get_stockfish();
    let mut current_elo = 1350u32;
    // let mut games_played = 0u32;

    loop {
        let stockfish = ChessBot {
            name: format!("{}_{}", stockfish.name.clone(), current_elo),
            path: stockfish.path.clone(),
        };

        println!("Playing games at elo {}", current_elo);

        let result = run_match(bot.clone(), &stockfish, current_elo);
        // games_played += result.len() as u32;

        // Check for two consecutive losses
        let consecutive_losses = result
            .windows(2)
            .any(|w| w[0] == GameResult::Loss && w[1] == GameResult::Loss);

        if consecutive_losses {
            cprintln!("<r,bold>Lost both games</>");
            cprintln!("<g,bold>Final elo: {}</>", current_elo - 50);
            return;
        } else {
            let last_two = result.windows(2).last().unwrap();
            cprintln!(
                "<c,bold>Elo {}: {} {}",
                current_elo,
                last_two[0],
                last_two[1]
            );
        }

        current_elo += 50;
    }
}

fn run_match(bot: ChessBot, stockfish: &ChessBot, elo: u32) -> Vec<GameResult> {
    let pgn_path = format!("game_{}.pgn", elo);

    let dir = PathBuf::from_str(".").unwrap().canonicalize().unwrap();

    let output = Command::new("fastchess")
        .arg("-engine")
        .arg(format!("cmd={}", bot.path))
        .arg(format!("name={}", bot.name))
        .arg("-engine")
        .arg(format!("cmd={}", stockfish.path))
        .arg(format!("name={}", stockfish.name))
        .arg("option.UCI_LimitStrength=true")
        .arg(format!("option.UCI_Elo={}", elo))
        .arg("-each")
        .arg("tc=300+0")
        .arg("proto=uci")
        .arg(format!("dir={}", dir.display()))
        .arg("-games")
        .arg("1")
        .arg("-pgnout")
        .arg(&pgn_path)
        .output()
        .expect("Failed to run fastchess");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("stdout: {}", stdout);
    parse_results(&stdout, &bot.name)
}

#[derive(Debug, PartialEq, Clone)]
enum GameResult {
    Win,
    Loss,
    Draw,
}

impl Display for GameResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResult::Win => {
                cwrite!(f, "<g,bold>Win</>")
            }
            GameResult::Loss => {
                cwrite!(f, "<y,bold>Loss</>")
            }
            GameResult::Draw => {
                cwrite!(f, "<r,bold>Loss</>")
            }
        }
    }
}

fn parse_results(output: &str, bot_name: &str) -> Vec<GameResult> {
    let mut results = Vec::new();

    for line in output.lines() {
        if !line.contains("Finished game") {
            continue;
        }

        // Determine which side our bot was on
        // Format: "Finished game N (White vs Black): SCORE {reason}"
        let bot_is_white = line
            .split('(')
            .nth(1)
            .and_then(|s| s.split(')').next())
            .map(|players| {
                let white = players.split(" vs ").next().unwrap_or("").trim();
                white == bot_name
            })
            .unwrap_or(false);

        // Extract score like "1-0", "0-1", "1/2-1/2"
        let score = line
            .split("): ")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .unwrap_or("");

        let result = match (score, bot_is_white) {
            ("1-0", true) | ("0-1", false) => GameResult::Win,
            ("0-1", true) | ("1-0", false) => GameResult::Loss,
            _ => GameResult::Draw,
        };

        results.push(result);
    }

    results
}
