//! No member of a server's process group outlives its `Session` (B1 family
//! "child group outlives its owner"): not after `stop`, not after a drop
//! without `stop`, also when a member ignores SIGTERM.
#![cfg(test)]

use std::ffi::OsString;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use bb_mcp::Session;

/// A server that forks a child which ignores TERM and holds no pipe, then
/// answers the handshake. `$exit_on_eof` picks how the leader ends.
fn script(exit_on_eof: bool) -> String {
    let tail: &str = if exit_on_eof {
        "exit 0;"
    } else {
        "$SIG{TERM} = 'IGNORE'; sleep 60;"
    };
    format!(
        r#"$| = 1;
if (fork() == 0) {{ $SIG{{TERM}} = 'IGNORE'; close STDIN; close STDOUT; close STDERR; sleep 60; exit 0; }}
my $init = <STDIN>;
my ($id) = $init =~ /"id":(\d+)/;
print qq({{"jsonrpc":"2.0","id":$id,"result":{{"protocolVersion":"2025-03-26","capabilities":{{}},"serverInfo":{{"name":"p","version":"1"}}}}}}\n);
while (my $line = <STDIN>) {{ }}
{tail}
"#
    )
}

async fn launch(exit_on_eof: bool) -> Session {
    let args: Vec<OsString> = vec!["-e".into(), script(exit_on_eof).into()];
    Session::launch("perl", &args).await.unwrap()
}

fn group_alive(group: u32) -> bool {
    Command::new("pgrep")
        .args(["-g", &group.to_string()])
        .stdout(Stdio::null())
        .status()
        .unwrap()
        .success()
}

/// Whether the group is gone within 1 s; a red run is cleaned up.
fn gone_soon(group: u32) -> bool {
    let started: Instant = Instant::now();
    while started.elapsed() < Duration::from_secs(1) {
        if !group_alive(group) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let _killed: std::io::Result<std::process::ExitStatus> = Command::new("/bin/kill")
        .args(["-KILL", "--", &format!("-{group}")])
        .status();
    false
}

#[tokio::test]
async fn stop_ends_members_that_outlive_the_leader() {
    let session: Session = launch(true).await;
    let group: u32 = session.pid().unwrap();
    assert!(session.stop().await.unwrap().success());
    assert!(gone_soon(group), "a TERM-ignoring member outlived stop");
}

#[tokio::test]
async fn dropping_a_session_without_stop_ends_its_whole_group() {
    let session: Session = launch(false).await;
    let group: u32 = session.pid().unwrap();
    drop(session);
    assert!(gone_soon(group), "the group outlived a dropped Session");
}

#[tokio::test]
async fn ending_all_servers_ends_every_live_group() {
    let first: Session = launch(false).await;
    let second: Session = launch(false).await;
    let groups: [u32; 2] = [first.pid().unwrap(), second.pid().unwrap()];
    bb_mcp::end_all_servers();
    for group in groups {
        assert!(gone_soon(group), "group {group} outlived end_all_servers");
    }
    drop((first, second));
}
