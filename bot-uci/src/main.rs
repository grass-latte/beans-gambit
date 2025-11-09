pub mod uci_state;

use crate::uci_state::{UciOptions, slow_uci_state, slow_uci_state_mut};
use std::borrow::Borrow;
use std::io::BufRead;
use std::time::Duration;
use std::{io, thread};
use strum::IntoEnumIterator;
use vampirc_uci::{UciInfoAttribute, UciMessage, parse_one};

fn send_uci<M: Borrow<UciMessage>>(msg: M) {
    println!("{}", msg.borrow());
}

fn send_info<S1: AsRef<str>, S2: AsRef<str>>(info: S1, value: S2) {
    println!(
        "{}",
        UciMessage::Info(vec![UciInfoAttribute::Any(
            info.as_ref().to_string(),
            value.as_ref().to_string()
        )])
    );
}

fn main() {
    for line in io::stdin().lock().lines() {
        let msg: UciMessage = parse_one(&line.unwrap());

        match msg {
            UciMessage::Uci => {
                // Id
                send_uci(UciMessage::Id {
                    name: Some("beans-gambit".to_string()),
                    author: Some("Robert Lucas / Benjamin Stott".to_string()),
                });
                // Options
                for option in UciOptions::iter() {
                    send_uci(UciMessage::Option(option.get_type()));
                }
                // Ok
                send_uci(UciMessage::UciOk);
            }
            UciMessage::Debug(debug) => slow_uci_state_mut().debug = debug,
            UciMessage::IsReady => {
                while !slow_uci_state().is_ready() {
                    thread::sleep(Duration::from_millis(5));
                }
                send_uci(UciMessage::ReadyOk);
            }
            UciMessage::SetOption { name, value } => {
                if let Some(value) = value {
                    slow_uci_state_mut().set_option_named(name, value).ok();
                } else {
                    slow_uci_state_mut().unset_option_named(name).ok();
                }
            }
            UciMessage::Register { later, name, code } => {
                // TODO
            }
            UciMessage::UciNewGame => {
                // TODO
            }
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => {
                // TODO
            }
            UciMessage::Go {
                time_control,
                search_control,
            } => {
                // TODO
            }
            UciMessage::Stop => {
                // TODO
            }
            UciMessage::PonderHit => {
                // TODO
            }
            UciMessage::Quit => {
                // TODO
            }
            UciMessage::Unknown(_, _) => {}
            _ => {}
        };
    }
}
