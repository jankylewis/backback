//! BBEOD29: Rust and Python windows on the REAL servers, by mouse and keys
//! (F12, Cmd+click, hover, F2, Problems); variant aware like the session
//! tests, and a failure on a missing server when `BACKBACK_REQUIRE_TOOLS`.

pub(crate) mod bb_kit;
mod python;
mod roslyn;
mod rust;

use std::time::{Duration, Instant};

use gpui::TestAppContext;
use tokio::sync::mpsc::error::TryRecvError;

use bb_kit::{Bridge, Live};

/// The servers a window test starts, as `ps` names them.
const SERVERS: [&str; 3] = ["LanguageServer", "rust-analyzer", "pyright-langserver"];

/// Close every window, wait until each server thread has ended (its event
/// channel closes only then) and find no server of this process left. A
/// test that returns sooner leaves its server holding the run's output
/// pipe: nextest's LEAK (BBEOD42).
pub(crate) fn end(live: Live, cx: &mut TestAppContext) {
    let Live { bridges, .. }: Live = live;
    super::super::close_all(cx);
    let started: Instant = Instant::now();
    for (from, _) in bridges.borrow_mut().iter_mut() {
        while !matches!(from.try_recv(), Err(TryRecvError::Disconnected)) {
            assert!(
                started.elapsed() < Duration::from_secs(60),
                "a server thread outlived its window"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    let out: std::process::Output = std::process::Command::new("/bin/ps")
        .args(["-A", "-ww", "-o", "ppid=,stat=,command="])
        .output()
        .unwrap();
    let own: String = std::process::id().to_string();
    let left: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|line: &&str| line.split_whitespace().next() == Some(own.as_str()))
        .filter(|line: &&str| {
            !line
                .split_whitespace()
                .nth(1)
                .is_some_and(|s: &str| s.starts_with('Z'))
        })
        .filter(|line: &&str| SERVERS.iter().any(|name: &&str| line.contains(name)))
        .map(str::to_owned)
        .collect();
    let _bridges: Vec<Bridge> = bridges.take();
    assert!(left.is_empty(), "{left:?}");
}
