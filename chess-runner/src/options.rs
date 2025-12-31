use crate::run::{ChessOptions, MatchSetup};
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use std::str::FromStr;
use strum::IntoEnumIterator;

pub fn select_options() -> ChessOptions {
    let items: Vec<_> = MatchSetup::iter().map(|a| a.to_string()).collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose one")
        .items(&items)
        .interact()
        .unwrap();
    let selection = MatchSetup::from_str(&items[selection]).unwrap();

    ChessOptions::new(selection)
}
