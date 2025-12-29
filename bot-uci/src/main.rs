#![allow(dead_code)]
#![allow(unused)]

pub mod uci_state;

use crate::uci_state::{UciOptions, slow_uci_state, slow_uci_state_mut};
use chrono::Local;
use fern::Dispatch;
use log::{debug, info};
use std::borrow::Borrow;
use std::fs::File;
use std::io::BufRead;
use std::time::Duration;
use std::{io, thread};
use strum::IntoEnumIterator;
use vampirc_uci::Rule::debug;
use vampirc_uci::{UciInfoAttribute, UciMessage, parse_one};

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn send_uci<M: Borrow<UciMessage>>(msg: M) {
    info!("Sending message {}", msg.borrow());
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
    let dispatch = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .chain(
            fern::log_file(format!(
                "bot-uci_{}.log",
                Local::now().format("%Y-%m-%d_%H-%M-%S_%6f")
            ))
            .unwrap(),
        );

    if cfg!(debug_assertions) {
        dispatch.level(log::LevelFilter::Debug).apply().unwrap();
    } else {
        dispatch.level(log::LevelFilter::Info).apply().unwrap();
    }

    println!(
        "Beans Gambit UCI v{} [Bot v{} | Chess Lib v{}]",
        version(),
        bot::version(),
        chess_lib::version()
    );

    info!("Waiting for stdin");

    for line in io::stdin().lock().lines() {
        let msg: UciMessage = parse_one(&line.unwrap());

        info!("Received message: {}", msg);

        match msg {
            UciMessage::Uci => {
                // Id
                send_uci(UciMessage::Id {
                    name: Some("beans-gambit".to_string()),
                    author: None,
                });
                send_uci(UciMessage::Id {
                    name: None,
                    author: Some("Robert Lucas / Benjamin Stott".to_string()),
                });
                // Options
                for option in UciOptions::iter() {
                    send_uci(UciMessage::Option(option.get_type()));
                }
                // Ok
                send_uci(UciMessage::UciOk);
            }
            UciMessage::Debug(is_debug) => slow_uci_state_mut().debug = is_debug,
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
