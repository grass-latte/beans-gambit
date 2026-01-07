use crate::run::{ChessBot, ChessOptions, MatchType};
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use std::collections::HashSet;
use std::str::FromStr;
use strum::IntoEnumIterator;

fn select_bot() -> (ChessBot, Option<String>) {
    todo!()
}

fn obtain_beans_versions(beans_versions: &HashSet<String>) {
    todo!()
}

pub fn select_options() -> ChessOptions {
    let items: Vec<_> = MatchType::iter().map(|a| a.to_string()).collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select option")
        .items(&items)
        .interact()
        .unwrap();
    let selection = MatchType::from_str(&items[selection]).unwrap();

    let mut bots = Vec::new();
    let mut beans_versions = HashSet::new();

    for _ in 0..selection.bots_required() {
        let (bot, beans_version) = select_bot();
        bots.push(bot);

        if let Some(beans_version) = beans_version {
            beans_versions.insert(beans_version);
        }
    }

    obtain_beans_versions(&beans_versions);

    ChessOptions::new(selection, bots)
}
