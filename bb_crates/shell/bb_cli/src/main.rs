//! `bb`: the backback command line. Until the GUI lands (M1) this is the proof
//! path and the readback tool: run it on real solutions, find what breaks.

mod cli;
mod commands;

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command: cli::Command = match cli::parse(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("bb: {e}\n\n{}", cli::USAGE);
            return ExitCode::from(2);
        }
    };
    match commands::run(command).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("bb: {e}");
            ExitCode::FAILURE
        }
    }
}
