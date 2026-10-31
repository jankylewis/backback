//! Bench command arguments.

use super::{CliError, Command, DEFAULT_RUNS, MAX_RUNS};
use std::path::PathBuf;

/// `bench [solution] [--runs N]`, in that order.
pub(super) fn parse(rest: &[String]) -> Result<Command, CliError> {
    let (solution, flags): (Option<PathBuf>, &[String]) = match rest.split_first() {
        Some((first, after)) if first != "--runs" => (Some(PathBuf::from(first)), after),
        Some(_) | None => (None, rest),
    };
    let runs: usize = match flags {
        [] => DEFAULT_RUNS,
        [flag, n] if flag == "--runs" => n
            .parse::<usize>()
            .ok()
            .filter(|r: &usize| (1..=MAX_RUNS).contains(r))
            .ok_or_else(|| CliError::BadRuns(n.clone()))?,
        [flag] if flag == "--runs" => return Err(CliError::BadRuns(String::new())),
        [.., last] => return Err(CliError::Extra(last.clone())),
    };
    Ok(Command::Bench { solution, runs })
}
