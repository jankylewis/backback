use super::*;

#[test]
fn parses_debug_breakpoint_and_rejects_bad_line() {
    assert_eq!(
        parse(&args(&["debug", "App.csproj", "--break", "Program.cs:12"])),
        Ok(Command::Debug {
            target: PathBuf::from("App.csproj"),
            source: PathBuf::from("Program.cs"),
            line: 12,
        })
    );
    assert_eq!(
        parse(&args(&["debug", "App.dll", "--break", "Program.cs:0"])),
        Err(CliError::BadBreak)
    );
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s: &&str| (*s).to_owned()).collect()
}

#[test]
fn parses_each_command() {
    assert_eq!(parse(&args(&["doctor"])), Ok(Command::Doctor));
    assert_eq!(
        parse(&args(&["scan", "A.slnx"])),
        Ok(Command::Scan {
            solution: PathBuf::from("A.slnx")
        })
    );
    assert_eq!(
        parse(&args(&["open", "A.slnx"])),
        Ok(Command::Open {
            solution: PathBuf::from("A.slnx"),
            restore: Restore::Off
        })
    );
    assert_eq!(
        parse(&args(&["open", "A.slnx", "--restore"])),
        Ok(Command::Open {
            solution: PathBuf::from("A.slnx"),
            restore: Restore::On
        })
    );
    assert_eq!(
        parse(&args(&["outline", "A.cs"])),
        Ok(Command::Outline {
            file: PathBuf::from("A.cs")
        })
    );
    assert_eq!(
        parse(&args(&["audit"])),
        Ok(Command::Audit {
            root: PathBuf::from(".")
        })
    );
}

#[test]
fn parses_audit_and_bench() {
    assert_eq!(
        parse(&args(&["bench"])),
        Ok(Command::Bench {
            solution: None,
            runs: DEFAULT_RUNS
        })
    );
    assert_eq!(
        parse(&args(&["bench", "A.slnx"])),
        Ok(Command::Bench {
            solution: Some(PathBuf::from("A.slnx")),
            runs: DEFAULT_RUNS
        })
    );
    assert_eq!(
        parse(&args(&["audit", "src"])),
        Ok(Command::Audit {
            root: PathBuf::from("src")
        })
    );
}

#[test]
fn rejects_bad_input() {
    assert_eq!(parse(&args(&["open"])), Err(CliError::NeedsPath("open")));
    assert_eq!(parse(&args(&["scan"])), Err(CliError::NeedsPath("scan")));
    assert_eq!(parse(&args(&[])), Err(CliError::Missing));
    assert_eq!(
        parse(&args(&["fly"])),
        Err(CliError::Unknown("fly".to_owned()))
    );
}

#[test]
fn rejects_arguments_a_command_does_not_take() {
    let extra: Result<Command, CliError> = Err(CliError::Extra("x".to_owned()));
    assert_eq!(parse(&args(&["doctor", "x"])), extra);
    assert_eq!(parse(&args(&["scan", "A.slnx", "x"])), extra);
    assert_eq!(parse(&args(&["open", "A.slnx", "x"])), extra);
    assert_eq!(parse(&args(&["open", "A.slnx", "--restore", "x"])), extra);
    assert_eq!(parse(&args(&["outline", "A.cs", "x"])), extra);
    assert_eq!(parse(&args(&["audit", "src", "x"])), extra);
    assert_eq!(parse(&args(&["bench", "A.slnx", "x"])), extra);
    assert_eq!(
        parse(&args(&["bench", "A.slnx", "--runs", "7"])),
        Ok(Command::Bench {
            solution: Some(PathBuf::from("A.slnx")),
            runs: 7
        })
    );
    assert_eq!(
        parse(&args(&["bench", "--runs", "0"])),
        Err(CliError::BadRuns("0".to_owned()))
    );
}
