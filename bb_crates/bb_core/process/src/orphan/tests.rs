use std::io::Write;
use std::num::NonZeroU32;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{Duration, Instant};

use super::bb_guard::{forget_line, watch_line, SCRIPT, SENT};
use super::records::{decode, encode, sweep, write, Record, Swept};
use super::started;
use crate::bb_group::GroupId;

const NEVER: &str = "Mon_Jan_1_00:00:00_2001";

fn server(leader_exits: bool) -> u32 {
    let script: &str = if leader_exits {
        "/bin/sleep 300 & exit 0"
    } else {
        "/bin/sleep 300 & wait"
    };
    let mut child: Child = Command::new("/bin/sh")
        .args(["-c", script])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .unwrap();
    let group: u32 = child.id();
    if leader_exits {
        child.wait().unwrap();
    } else {
        let _reaper: std::thread::JoinHandle<()> = std::thread::spawn(move || {
            let _status: std::io::Result<std::process::ExitStatus> = child.wait();
        });
    }
    std::thread::sleep(Duration::from_millis(100));
    assert!(alive(group), "the stand-in started");
    group
}

fn token(group: u32) -> String {
    started(group).unwrap_or_else(|| String::from("-"))
}

fn alive(group: u32) -> bool {
    GroupId::foreign(NonZeroU32::new(group).unwrap()).alive()
}

fn gone_within(group: u32, within: Duration) -> bool {
    let started: Instant = Instant::now();
    while started.elapsed() < within {
        if !alive(group) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

fn end(groups: &[u32]) {
    for group in groups {
        GroupId::foreign(NonZeroU32::new(*group).unwrap()).end_now();
    }
}

fn guard() -> (Child, ChildStdin) {
    let mut child: Child = Command::new("/bin/sh")
        .args(["-c", SCRIPT, "bb-guard"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .unwrap();
    let pipe: ChildStdin = child.stdin.take().unwrap();
    (child, pipe)
}

#[test]
fn the_guard_ends_its_watched_groups_when_the_pipe_closes_and_spares_the_rest() {
    let watched: u32 = server(false);
    let headless: u32 = server(true);
    let forgotten: u32 = server(false);
    let reused: u32 = server(false);
    let stranger: u32 = server(false);
    let (mut child, mut pipe): (Child, ChildStdin) = guard();
    for group in [watched, headless, forgotten] {
        pipe.write_all(watch_line(group, &token(group)).as_bytes())
            .unwrap();
    }
    pipe.write_all(watch_line(reused, NEVER).as_bytes())
        .unwrap();
    pipe.write_all(forget_line(forgotten).as_bytes()).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    assert!(alive(watched), "nothing ends while the pipe is open");
    drop(pipe);
    let left: Vec<(u32, bool)> = vec![
        (watched, gone_within(watched, Duration::from_secs(3))),
        (headless, gone_within(headless, Duration::from_secs(3))),
        (forgotten, alive(forgotten)),
        (reused, alive(reused)),
        (stranger, alive(stranger)),
    ];
    child.wait().unwrap();
    end(&[forgotten, reused, stranger, watched, headless]);
    assert!(
        left.iter().all(|(_, ok): &(u32, bool)| *ok),
        "watched, headless gone; forgotten, reused, stranger kept: {left:?}"
    );
}

#[test]
fn records_round_trip_and_an_unknown_one_is_refused() {
    let record: Record = Record {
        owner: (7, String::from("Fri_Nov_20_21:16:09_2026")),
        groups: vec![(9, String::from("-")), (11, String::from(NEVER))],
    };
    assert_eq!(decode(&encode(&record)), Some(record));
    assert_eq!(decode("backback children v2\nowner 7 x\n"), None);
    assert_eq!(decode("backback children v1\nowner 7\n"), None);
    assert_eq!(
        decode("backback children v1\nowner 7 x\ngroup nine x\n"),
        None
    );
    assert_eq!(decode(""), None);
}

fn folder(name: &str) -> PathBuf {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("bb-process-{name}-{}", std::process::id()));
    let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn record(dir: &Path, owner: (u32, &str), groups: &[(u32, String)]) {
    write(
        dir,
        &Record {
            owner: (owner.0, owner.1.to_owned()),
            groups: groups.to_vec(),
        },
    );
}

fn unknown_record(dir: &Path, group: u32) -> PathBuf {
    let file: PathBuf = dir.join("99999997");
    std::fs::write(&file, format!("backback children v9\ngroup {group} -\n")).unwrap();
    file
}

#[test]
fn a_launch_sweep_ends_only_what_a_dead_backback_started() {
    let dir: PathBuf = folder("sweep");
    let left: u32 = server(false);
    let headless: u32 = server(true);
    let reused: u32 = server(false);
    let running: u32 = server(false);
    let stranger: u32 = server(false);
    let unknown_group: u32 = server(false);
    record(
        &dir,
        (99_999_998, "Thu_Nov_19_22:00:00_2026"),
        &[
            (left, token(left)),
            (headless, token(headless)),
            (reused, String::from(NEVER)),
        ],
    );
    let own: u32 = std::process::id();
    record(&dir, (own, &token(own)), &[(running, token(running))]);
    let unknown_file: PathBuf = unknown_record(&dir, unknown_group);
    let swept: Swept = sweep(&dir);
    let kept: Vec<(u32, bool)> = vec![
        (left, !alive(left)),
        (headless, !alive(headless)),
        (reused, alive(reused)),
        (running, alive(running)),
        (stranger, alive(stranger)),
        (unknown_group, alive(unknown_group)),
    ];
    let files: (bool, bool, bool) = (
        dir.join("99999998").exists(),
        dir.join(own.to_string()).exists(),
        unknown_file.exists(),
    );
    end(&[left, headless, reused, running, stranger, unknown_group]);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(swept.ended, vec![left, headless], "{kept:?}");
    assert_eq!(swept.refused, vec![unknown_file]);
    assert!(kept.iter().all(|(_, ok): &(u32, bool)| *ok), "{kept:?}");
    assert_eq!(
        files,
        (false, true, true),
        "dead owner's file gone; live and unknown kept"
    );
}

#[test]
fn an_ended_group_is_told_to_the_guard_so_a_reused_pgid_is_never_its_target() {
    let group: u32 = server(false);
    let owned: crate::OwnedGroup = crate::OwnedGroup::register(NonZeroU32::new(group).unwrap());
    drop(owned);
    let told: Vec<String> = SENT.lock().unwrap().clone();
    let watched_at: Option<usize> = told
        .iter()
        .position(|l: &String| l.starts_with(&format!("+ {group} ")));
    let forgotten_at: Option<usize> = told.iter().position(|l: &String| *l == forget_line(group));
    assert!(
        watched_at.is_some() && forgotten_at > watched_at,
        "{told:?}"
    );
}
