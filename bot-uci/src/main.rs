pub mod state;

use crate::state::{slow_global_state, slow_global_state_mut};
use std::borrow::Borrow;
use std::{io, thread};
use std::io::BufRead;
use std::time::Duration;
use vampirc_uci::{parse_one, UciMessage};

fn send_uci<M: Borrow<UciMessage>>(msg: M) {
    println!("{}", msg.borrow());
}


fn main() {
    for line in io::stdin().lock().lines() {
        let msg: UciMessage = parse_one(&line.unwrap());

        match msg {
            UciMessage::Uci => {
                send_uci(UciMessage::Id {
                    name: Some("beans-gambit".to_string()),
                    author: Some("Robert Lucas / Benjamin Stott".to_string()),
                });
                send_uci(UciMessage::UciOk);
            }
            UciMessage::Debug(debug) => {
                slow_global_state_mut().debug = debug
            }
            UciMessage::IsReady => {
                while !slow_global_state().is_ready() {
                    thread::sleep(Duration::from_millis(5));
                }
                send_uci(UciMessage::ReadyOk);
            }
            UciMessage::SetOption {name, value} => {

                slow_global_state_mut().set_option_named(name, value).ok();
            }
            UciMessage::Unknown(_, _) => {}
            _ => {}
        };
    }
}
