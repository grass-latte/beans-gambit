use crate::setup::{BotVsBotOptions, ChessBot, ChessOptions, MatchType, PerformanceOptions};
use color_print::{cformat, cprintln};
use itertools::Itertools;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::FromStr;

#[allow(unused)]
fn bot_vs_bot(bot1: ChessBot, bot2: ChessBot, options: &BotVsBotOptions) {
    let mut command = Command::new("fastchess");
    let command = command
        .arg("-engine")
        .arg(format!("cmd={}", &bot1.path))
        .arg(format!("name={}", &bot1.name))
        .arg("-engine")
        .arg(format!("cmd={}", &bot2.path))
        .arg(format!("name={}", &bot2.name))
        .arg("-each")
        .arg("tc=30+0")
        .arg("proto=uci")
        .arg(format!(
            "dir={}",
            PathBuf::from_str(".")
                .unwrap()
                .canonicalize()
                .unwrap()
                .display()
        ))
        .arg("-games")
        .arg(options.games.to_string())
        .arg("-pgnout")
        .arg("game.pgn");

    cprintln!(
        "<c>Args: {}</>",
        command
            .get_args()
            .map(|s| s.to_string_lossy())
            .collect_vec()
            .join(" ")
    );

    let Ok(status) = command.status() else {
        panic!("{}", cformat!("<r,bold>Failed to run fastchess</>"));
    };

    if status.success() {
        cprintln!("<g,bold>Match finished successfully!</>");
    } else {
        panic!(
            "{}",
            cformat!("<r,bold>fastchess exited with code: {:?}</>", status.code())
        );
    }
}

fn compliance(bot: ChessBot) {
    let mut command = Command::new("fastchess");
    let command = command.arg("--compliance").arg(bot.path);

    cprintln!(
        "<c>Args: {}</>",
        command
            .get_args()
            .map(|s| s.to_string_lossy())
            .collect_vec()
            .join(" ")
    );

    let Ok(status) = command.status() else {
        panic!("{}", cformat!("<r,bold>Failed to run fastchess</>"));
    };

    if status.success() {
        cprintln!("<g,bold>Tool ran successfully!</>");
    } else {
        panic!(
            "{}",
            cformat!("<r,bold>fastchess exited with code: {:?}</>", status.code())
        );
    }
}

fn wait_for(reader: &mut BufReader<std::process::ChildStdout>, token: &str) {
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).unwrap();

        if n == 0 {
            panic!("Process ended early");
        }

        if line.contains(token) {
            break;
        }
    }
}

// #[cfg(unix)]
// fn send_ctrl_c(pid: u32) {
//     use nix::sys::signal::{kill, Signal};
//     use nix::unistd::Pid;
//
//     kill(Pid::from_raw(pid as i32), Signal::SIGINT).unwrap();
// }
//
// #[cfg(windows)]
// fn send_ctrl_c(pgid: u32) {
//     use windows_sys::Win32::System::Console::GenerateConsoleCtrlEvent;
//     use windows_sys::Win32::System::Console::CTRL_C_EVENT;
//     use windows_sys::Win32::System::Console::CTRL_BREAK_EVENT;
//
//     let ok = unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pgid) };
//     if ok == 0 {
//         eprintln!("GenerateConsoleCtrlEvent failed");
//     }
// }

fn performance(bot: ChessBot, options: &PerformanceOptions) {
    let mut command = Command::new("flamegraph");
    let command = command
        .arg("--")
        .arg(bot.path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    cprintln!(
        "<c>Flamegraph args: {}</>",
        command
            .get_args()
            .map(|s| s.to_string_lossy())
            .collect_vec()
            .join(" ")
    );

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }

    let Ok(mut child) = command.spawn() else {
        panic!(
            "{}",
            cformat!("<r,bold>Failed to run flamegraph on binary</>")
        );
    };

    // let pid = child.id();
    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    println!("Performing UCI handshake");

    print!("1: uci");
    writeln!(stdin, "uci").unwrap();
    stdin.flush().unwrap();
    wait_for(&mut reader, "uciok");

    print!("\r2: isready");
    writeln!(stdin, "isready").unwrap();
    stdin.flush().unwrap();
    wait_for(&mut reader, "readyok");

    print!("\r3: ucinewgame");
    writeln!(stdin, "ucinewgame").unwrap();
    stdin.flush().unwrap();

    let uci_cmd = format!("position fen {}", options.fen);
    print!("\r4: {}", &uci_cmd);
    writeln!(stdin, "{}", uci_cmd).unwrap();
    stdin.flush().unwrap();

    println!("\nStarting search: go wtime 300000 btime 300000 movestogo 40");
    writeln!(stdin, "go wtime 300000 btime 300000 movestogo 40").unwrap();
    stdin.flush().unwrap();

    wait_for(&mut reader, "bestmove");

    println!("Search complete, sending quit");
    writeln!(stdin, "quit").unwrap();
    stdin.flush().unwrap();

    println!("Waiting for exit");
    let _ = child.wait();

    println!("Opening flamegraph...");
    if let Err(e) = open::that("flamegraph.svg") {
        cprintln!("<r,bold>Failed to open flamegraph: {e:?}</>")
    }
}

fn cutechess() {
    #[cfg(unix)]
    let mut command = Command::new("cutechess");
    #[cfg(windows)]
    let mut command = Command::new("cutechess.exe");

    match command.status() {
        Ok(s) if s.success() => {}
        Ok(s) => {
            cprintln!("<r,bold>cutechess exited with bad status: {s:?}</>")
        }
        Err(e) => {
            cprintln!("<r,bold>Failed to run cutechess: {e:?}</>")
        }
    }
}

pub fn run(options: ChessOptions, bots: Vec<ChessBot>) {
    match options.setup() {
        MatchType::BuildAndRunCutechess => cutechess(),
        MatchType::Compliance => compliance(bots[0].clone()),
        MatchType::Performance(options) => performance(bots[0].clone(), options),
        MatchType::BotVsBot(options) => bot_vs_bot(bots[0].clone(), bots[1].clone(), options),
        MatchType::BuildOnly => {
            println!("Bot {} at {}", &bots[0].name, &bots[0].path);
        }
        MatchType::BuildAndRun => {
            Command::new(&bots[0].path).status().unwrap();
        }
    }
}
