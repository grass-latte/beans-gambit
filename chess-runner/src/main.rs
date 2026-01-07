mod bot_resolver;
mod run;
pub mod setup;

use crate::bot_resolver::resolve_bot;
use crate::setup::select_options;
use itertools::Itertools;
use run::run;
use std::collections::HashSet;

fn main() {
    let (options, unresolved_bots) = select_options();

    let mut bots = unresolved_bots.into_iter().map(resolve_bot).collect_vec();

    // Ensure unique bot names
    let mut bot_names = HashSet::new();
    for bot in &mut bots {
        let mut suffix = 2;
        if bot_names.insert(bot.name.clone()) {
            continue;
        }

        while !bot_names.insert(format!("{}_{}", bot.name, suffix)) {
            suffix += 1;
        }

        bot.name = format!("{}_{}", bot.name, suffix);
    }

    run(options, bots);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot_resolver::resolve_local_bot;
    use crate::run::{ChessOptions, MatchType};
    use crate::setup::LocalBot;

    #[test]
    fn test_fastchess_compliance() {
        let bots = vec![resolve_local_bot(LocalBot::BeansGambitLocal)];
        let options = ChessOptions::new(MatchType::Compliance);
        run(options, bots);
    }

    // #[test]
    // fn test_bot_vs_bot() {
    //     let bots = vec![
    //         resolve_local_bot(LocalBot::BeansGambitLocal),
    //         resolve_local_bot(LocalBot::BeansGambitLocal),
    //     ];
    //     let options = ChessOptions::new(MatchType::BotVsBot);
    //     run(options, bots);
    // }

    #[test]
    fn test_bot_vs_stockfish() {
        let bots = vec![
            resolve_local_bot(LocalBot::BeansGambitLocal),
            resolve_local_bot(LocalBot::Stockfish),
        ];
        let options = ChessOptions::new(MatchType::BotVsBot);
        run(options, bots);
    }
}
