#![allow(dead_code)]
#![allow(unused)]

mod uci_conversions;
mod uci_state;

use crate::uci_conversions::{from_uci_move, to_uci_piece, to_uci_square};
use crate::uci_state::{UciOptions, UciState};
use chess_lib::board::{Board, Move};
use chrono::Local;
use engine::{InterMoveCache, search};
use fern::Dispatch;
use log::{debug, error, info, warn};
use std::borrow::Borrow;
use std::fs::File;
use std::io::{BufRead, Write};
use std::ops::DerefMut;
use std::process::exit;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread};
use strum::IntoEnumIterator;
use vampirc_uci::{UciInfoAttribute, UciMessage, UciMove, UciPiece, UciSquare, parse_one};

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

static RUNNING: AtomicBool = AtomicBool::new(false);
static SHOULD_STOP: AtomicBool = AtomicBool::new(false);

fn main() {
    std::panic::set_hook(Box::new(|info| {
        log::error!("panic: {info}");
    }));

    let dispatch = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S.%9f"),
                record.level(),
                message
            ))
        })
        .chain(
            fern::log_file(format!(
                "engine-uci_{}.log",
                Local::now().format("%Y-%m-%d_%H-%M-%S_%6f")
            ))
            .unwrap(),
        );

    if cfg!(debug_assertions) {
        dispatch.level(log::LevelFilter::Debug).apply().unwrap();
    } else {
        dispatch.level(log::LevelFilter::Info).apply().unwrap();
    }

    info!(
        "Binary: {}",
        std::env::current_exe().unwrap().to_str().unwrap()
    );

    let mut debug = false;
    let mut state = UciState::new();
    let mut board = Board::starting();
    let cache = Arc::new(Mutex::new(InterMoveCache::new()));

    println!(
        "Beans Gambit UCI v{} [Bot v{} | Chess Lib v{}]",
        version(),
        engine::version(),
        chess_lib::version()
    );

    info!(
        "Beans Gambit UCI v{} [Bot v{} | Chess Lib v{}]",
        version(),
        engine::version(),
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
            UciMessage::Debug(is_debug) => debug = is_debug,
            UciMessage::IsReady => {
                send_uci(UciMessage::ReadyOk);
            }
            UciMessage::SetOption { name, value } => {
                if let Some(value) = value {
                    state.set_option_named(name, value).ok();
                } else {
                    state.unset_option_named(name).ok();
                }
            }
            UciMessage::Register { later, name, code } => {
                todo!()
            }
            UciMessage::UciNewGame => {
                // TODO
                *cache.lock().unwrap() = InterMoveCache::new();
            }
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => {
                if startpos {
                    board = Board::starting();
                } else {
                    board = Board::from_fen(fen.unwrap().as_str()).unwrap();
                }
                for m in moves {
                    let m = from_uci_move(m);
                    // TODO: Any move validation?
                    board.make_move(m);
                }
            }
            UciMessage::Go {
                time_control,
                search_control,
            } => {
                if RUNNING.load(Ordering::Acquire) {
                    panic!("Tried to go while already running!");
                }

                RUNNING.store(true, Ordering::Release);

                let cache = cache.clone();
                thread::spawn(move || {
                    let cache = cache;
                    let mut c = cache.lock().unwrap();
                    let mut board = board;
                    let result =
                        search(&mut board, &mut c, || SHOULD_STOP.load(Ordering::Acquire)).unwrap();

                    // fastchess requires at least one info message with score
                    send_uci(UciMessage::Info(vec![UciInfoAttribute::Score {
                        cp: Some(0),
                        mate: None,
                        lower_bound: None,
                        upper_bound: None,
                    }]));

                    send_uci(UciMessage::BestMove {
                        best_move: UciMove {
                            from: to_uci_square(result.source),
                            to: to_uci_square(result.destination),
                            promotion: result.promotion.map(to_uci_piece),
                        },
                        ponder: None,
                    });

                    RUNNING.store(false, Ordering::Release);
                });
            }
            UciMessage::Stop => {
                if RUNNING.load(Ordering::Acquire) {
                    info!("Setting stop flag");
                    SHOULD_STOP.store(true, Ordering::Release);
                    info!("Waiting for run flag to be unset");
                    while RUNNING.load(Ordering::Acquire) {
                        thread::sleep(Duration::from_millis(5));
                    }
                    info!("Unsetting stop flag");
                    SHOULD_STOP.store(false, Ordering::Release);
                }
            }
            UciMessage::PonderHit => {
                todo!()
            }
            UciMessage::Quit => {
                info!("Exiting on UCI Quit command");
                exit(0);
            }
            UciMessage::Unknown(_, _) => {
                warn!("Unknown UCI message: {:?}", msg);
            }
            _ => {
                warn!("Unhandled UCI message: {:?}", msg);
            }
        };
    }
}
