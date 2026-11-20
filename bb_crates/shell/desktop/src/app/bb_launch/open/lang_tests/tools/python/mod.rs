//! BBEOD28 in real windows: Python requests reach pyright only and the
//! missing-pyright hint (`intel`, fake servers); Build > Python Check and
//! the Tests panel on this machine's tools (`panels`). BBEOD41: Python is
//! opt-in; with no saved choice it starts nothing and shows nothing
//! (`opt_in`, `unpicked`). Here: the probe that tells a running pyright
//! server from any other process.

use std::path::Path;

#[path = "bb_intel.rs"]
mod intel;
mod opt_in;
mod panels;
mod unpicked;

/// The pyright language servers under this test process, read from `ps`.
/// Only the program counts, never its arguments: nextest runs each test as
/// `<test binary> --exact <full test name>`, and a test named for pyright
/// would otherwise see itself (BB-GAT-019). The test process is never one.
fn pyright_servers() -> Vec<String> {
    let out: std::process::Output = std::process::Command::new("ps")
        .args(["-A", "-o", "pid=,ppid=,command="])
        .output()
        .unwrap();
    let rows: Vec<(u32, u32, String)> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line: &str| {
            let mut parts: std::str::SplitWhitespace<'_> = line.split_whitespace();
            let child: u32 = parts.next()?.parse().ok()?;
            let parent: u32 = parts.next()?.parse().ok()?;
            Some((child, parent, parts.collect::<Vec<&str>>().join(" ")))
        })
        .collect();
    let me: u32 = std::process::id();
    let mut ours: Vec<u32> = vec![me];
    let mut grew: bool = true;
    while grew {
        let more: Vec<u32> = rows
            .iter()
            .filter(|(pid, ppid, _): &&(u32, u32, String)| {
                ours.contains(ppid) && !ours.contains(pid)
            })
            .map(|(pid, _, _): &(u32, u32, String)| *pid)
            .collect();
        grew = !more.is_empty();
        ours.extend(more);
    }
    rows.into_iter()
        .filter(|(pid, _, cmd): &(u32, u32, String)| {
            *pid != me && ours.contains(pid) && runs_a_pyright_server(cmd)
        })
        .map(|(pid, _, cmd): (u32, u32, String)| format!("{pid} {cmd}"))
        .collect()
}

/// `pyright-langserver --stdio`, or `node .../pyright-langserver --stdio`
/// as its `#!/usr/bin/env node` line runs it: the program, or the script
/// node runs, is a pyright server.
fn runs_a_pyright_server(command: &str) -> bool {
    let mut words: std::str::SplitWhitespace<'_> = command.split_whitespace();
    match (words.next(), words.next()) {
        (Some(program), _) if is_a_server(program) => true,
        (Some(program), Some(script)) => {
            Path::new(program).file_name() == Some(std::ffi::OsStr::new("node"))
                && is_a_server(script)
        }
        _ => false,
    }
}

fn is_a_server(word: &str) -> bool {
    let base: &str = Path::new(word)
        .file_name()
        .and_then(|n: &std::ffi::OsStr| n.to_str())
        .unwrap_or(word);
    bb_python::pyright::SERVERS.contains(&base)
}

#[test]
fn the_probe_sees_a_pyright_server_and_never_a_test_named_for_pyright() {
    let ci_row: &str = "/r/.target/debug/deps/backback-0319b12b1b43422c --exact \
        launch::open::lang_tests::tools::python::opt_in::\
        a_csharp_and_python_folder_with_no_saved_choice_starts_no_pyright_process --nocapture";
    assert!(!runs_a_pyright_server(ci_row));
    assert!(!runs_a_pyright_server(
        "/x/node_modules/.bin/pyright --version"
    ));
    assert!(!runs_a_pyright_server(
        "node /x/pyright-check.js pyright-langserver"
    ));
    assert!(runs_a_pyright_server(
        "node /opt/homebrew/bin/pyright-langserver --stdio"
    ));
    assert!(runs_a_pyright_server("/x/bin/pyright-langserver --stdio"));
    assert!(runs_a_pyright_server(
        "/x/bin/basedpyright-langserver --stdio"
    ));
}
