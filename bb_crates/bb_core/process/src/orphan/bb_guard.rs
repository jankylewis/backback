//! The guard: a tiny `/bin/sh` that ends this process's child groups when
//! this process ends in ANY way (BB-PRC-009). A drop, a quit hook or a
//! signal handler needs this process to run its cleanup code; a SIGKILL, an
//! abort (release builds abort on panic), `std::process::exit` or nextest
//! killing a timed-out test run none of it. The guard needs none of it: it
//! holds the read end of a pipe whose only write end is this process's, so
//! the kernel closes it however this process ends, and the guard's `read`
//! returns end of file.
//!
//! Each line on the pipe is `+ <group> <start>` (watch) or `- <group>`
//! (forget). At end of file the guard sends TERM to every group still
//! watched whose leader is the same process (`lstart` token) or gone while
//! members remain (Roslyn's `BuildHost` and MSBuild node, BBEOD42), waits up
//! to a second, then KILLs what is left. A pid reused by another program
//! never matches its token, so the guard never ends a group it did not see
//! start. It runs in its own process group (a terminal's Ctrl+C or nextest's
//! group kill do not reach it), ignores HUP/INT/QUIT/TERM, and has no
//! stdout or stderr (a test's output pipe is never held open by it).

use std::io::Write;
use std::os::unix::process::CommandExt;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// The guard's script; `$0` is `bb-guard` in `ps`.
pub(crate) const SCRIPT: &str = r#"trap '' HUP INT QUIT TERM PIPE
list=''
while IFS=' ' read -r op g s; do
  case $op in
    +) list="$list $g:$s" ;;
    -) keep=''
       for e in $list; do [ "${e%%:*}" = "$g" ] || keep="$keep $e"; done
       list=$keep ;;
  esac
done
hit=''
for e in $list; do
  g=${e%%:*}; s=${e#*:}
  set -- $(/bin/ps -o lstart= -p "$g" 2>/dev/null)
  IFS=_; now="$*"; IFS=' '
  if [ -z "$now" ]; then
    kill -0 -- "-$g" 2>/dev/null && hit="$hit $g"
  elif [ "$now" = "$s" ]; then
    hit="$hit $g"
  fi
done
[ -n "$hit" ] || exit 0
for g in $hit; do kill -TERM -- "-$g" 2>/dev/null; done
n=0
while [ $n -lt 10 ]; do
  left=''
  for g in $hit; do kill -0 -- "-$g" 2>/dev/null && left="$left $g"; done
  [ -n "$left" ] || exit 0
  /bin/sleep 0.1
  n=$((n+1))
done
for g in $hit; do kill -KILL -- "-$g" 2>/dev/null; done
"#;

/// The running guard and the write end of its pipe.
struct Guard {
    child: Child,
    pipe: ChildStdin,
}

static GUARD: Mutex<Option<Guard>> = Mutex::new(None);
/// Every line sent to the guard (tests read what the guard was told).
#[cfg(test)]
pub(crate) static SENT: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn guard() -> MutexGuard<'static, Option<Guard>> {
    GUARD.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Start the guard now when none runs (the app calls this first thing,
/// before any other thread can spawn a child that might inherit the pipe).
pub(crate) fn ensure(watched: &[(u32, String)]) {
    let mut slot: MutexGuard<'static, Option<Guard>> = guard();
    if slot.is_none() {
        *slot = start(watched);
    }
}

/// Send `line`; a guard that is gone (killed by hand) is started again and
/// told every group in `watched`, which already holds this change.
pub(crate) fn send(line: &str, watched: &[(u32, String)]) {
    #[cfg(test)]
    SENT.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(line.to_owned());
    let mut slot: MutexGuard<'static, Option<Guard>> = guard();
    let sent: bool = slot
        .as_mut()
        .is_some_and(|running: &mut Guard| running.pipe.write_all(line.as_bytes()).is_ok());
    if !sent {
        if let Some(mut gone) = slot.take() {
            let _reaped: std::io::Result<Option<std::process::ExitStatus>> = gone.child.try_wait();
        }
        *slot = start(watched);
    }
}

/// Close the pipe (app quit, after every group ended): the guard reads end
/// of file with nothing watched and exits; it is reaped here.
pub(crate) fn close() {
    if let Some(Guard { mut child, pipe }) = guard().take() {
        drop(pipe);
        let _reaped: std::io::Result<std::process::ExitStatus> = child.wait();
    }
}

/// The guard's pid, when one runs (tests).
pub(crate) fn pid() -> Option<u32> {
    guard().as_ref().map(|running: &Guard| running.child.id())
}

fn start(watched: &[(u32, String)]) -> Option<Guard> {
    let mut child: Child = Command::new("/bin/sh")
        .args(["-c", SCRIPT, "bb-guard"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .ok()?;
    let mut pipe: ChildStdin = child.stdin.take()?;
    for (group, started) in watched {
        let _told: std::io::Result<()> = pipe.write_all(watch_line(*group, started).as_bytes());
    }
    Some(Guard { child, pipe })
}

/// The line that watches `group`, whose leader started at `started`.
pub(crate) fn watch_line(group: u32, started: &str) -> String {
    format!("+ {group} {started}\n")
}

/// The line that forgets `group`.
pub(crate) fn forget_line(group: u32) -> String {
    format!("- {group}\n")
}
