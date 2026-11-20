//! Start a real `backback`, wait for the processes a scenario needs, end
//! it, and list what is left (BBEOD42).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use super::bb_tree::{fixture, launch, shown, tree, wanted, App, Row};
use super::hang;

const SERVERS_UP: Duration = Duration::from_secs(240);
const LEFT_AFTER: Duration = Duration::from_secs(5);

pub struct Running {
    pub app: App,
    pub pid: u32,
    seen: BTreeMap<u32, Row>,
    _dir: tempfile::TempDir,
}

pub fn running(keys: &str, quit: bool, mark: &str) -> Running {
    running_with(keys, quit, mark, &[])
}

pub fn running_with(keys: &str, quit: bool, mark: &str, also: &[&'static str]) -> Running {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let (file, _root, state): (PathBuf, PathBuf, PathBuf) = fixture(dir.path());
    let mut servers: Vec<&str> = wanted();
    servers.extend_from_slice(also);
    up(dir, (&file, &state), (keys, quit, mark), &servers)
}

fn up(
    dir: tempfile::TempDir,
    (file, state): (&Path, &Path),
    (keys, quit, mark): (&str, bool, &str),
    servers: &[&str],
) -> Running {
    let app: App = launch(file, state, keys, quit);
    let pid: u32 = app.child.id();
    let mut seen: BTreeMap<u32, Row> = BTreeMap::new();
    let started: Instant = Instant::now();
    let mut marked: bool = false;
    while started.elapsed() < SERVERS_UP {
        seen.extend(tree(pid, &seen));
        marked = marked || app.printed(mark, Duration::ZERO);
        let all_up: bool = servers
            .iter()
            .all(|name: &&str| seen.values().any(|row: &Row| row.command.contains(name)));
        if all_up && marked {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    let missing: Vec<&&str> = servers
        .iter()
        .filter(|name: &&&str| !seen.values().any(|row: &Row| row.command.contains(**name)))
        .collect();
    assert!(
        missing.is_empty() && marked,
        "servers missing {missing:?}, mark {mark} printed {marked}; said {:?}",
        app.said.lock().unwrap()
    );
    Running {
        app,
        pid,
        seen,
        _dir: dir,
    }
}

pub fn nothing_left(mut run: Running, how: &str) {
    if run.app.child.try_wait().unwrap().is_none() {
        let _ended: bool = run.app.ended_within(Duration::from_secs(30));
    }
    let _reaped: std::io::Result<std::process::ExitStatus> = run.app.child.wait();
    let before: Vec<String> = shown(&run.seen);
    let started: Instant = Instant::now();
    let mut left: BTreeMap<u32, Row> = tree(run.pid, &run.seen);
    while !left.is_empty() && started.elapsed() < LEFT_AFTER {
        std::thread::sleep(Duration::from_millis(200));
        run.seen.extend(left);
        left = tree(run.pid, &run.seen);
    }
    eprintln!(
        "{how}: before {before:#?}\n{how}: left after {LEFT_AFTER:?}: {:#?}",
        shown(&left)
    );
    let pids: Vec<String> = left.keys().map(u32::to_string).collect();
    if !pids.is_empty() {
        let _cleanup: std::io::Result<std::process::ExitStatus> =
            Command::new("/bin/kill").arg("-KILL").args(&pids).status();
    }
    assert!(left.is_empty(), "{how} left {:#?}", shown(&left));
}

pub fn building(keys: &str, mark: &str) -> Running {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let (file, state): (PathBuf, PathBuf) = hang::fixture(dir.path());
    up(dir, (&file, &state), (keys, false, mark), &[hang::STEP])
}
