//! `bb mcp` never leaves its server behind: a fatal signal to `bb` ends the
//! server's whole process group, also when the server ignores stdin EOF and
//! SIGTERM (B1, family "child group outlives its owner").

use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use crate::common::harness;

/// After the host stops (EOF or TERM), the server keeps running: only KILL
/// ends it.
const IGNORE_EOF_AND_TERM: &str = "await builder.Build().RunAsync();\n\
     await System.Threading.Tasks.Task.Delay(-1);";

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry: std::fs::DirEntry = entry.unwrap();
        let name: std::ffi::OsString = entry.file_name();
        if name == "bin" || name == "obj" {
            continue;
        }
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to.join(&name));
        } else {
            std::fs::copy(entry.path(), to.join(&name)).unwrap();
        }
    }
}

/// A fresh copy of the `_Harness` fixture whose MCP server ignores EOF and TERM.
fn stubborn_fixture() -> PathBuf {
    let made: PathBuf =
        std::env::temp_dir().join(format!("bb-mcp-stubborn-{}", std::process::id()));
    let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&made);
    copy_tree(&harness(), &made);
    let root: PathBuf = made.canonicalize().unwrap();
    let program: PathBuf = root.join("McpServer/Program.cs");
    let text: String = std::fs::read_to_string(&program).unwrap();
    assert!(text.contains("await builder.Build().RunAsync();"));
    let edited: String = text.replace("await builder.Build().RunAsync();", IGNORE_EOF_AND_TERM);
    std::fs::write(&program, edited).unwrap();
    root
}

/// The pid of the running server built from `root`, if any.
fn server_pid(root: &Path) -> Option<u32> {
    let pattern: String = format!("{}/McpServer/bin/.*/McpServer.dll", root.display());
    let out: std::process::Output = Command::new("pgrep")
        .args(["-f", &pattern])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(|line: &str| line.trim().parse::<u32>().ok())
}

fn group_alive(group: u32) -> bool {
    Command::new("pgrep")
        .args(["-g", &group.to_string()])
        .stdout(Stdio::null())
        .status()
        .unwrap()
        .success()
}

fn wait_until(deadline: Duration, mut done: impl FnMut() -> bool) -> bool {
    let started: Instant = Instant::now();
    while started.elapsed() < deadline {
        if done() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    done()
}

/// Run `bb <args>`, wait for its server, send `signal` to `bb` only, and
/// return `bb`'s exit status and whether the server group is gone 1 s later.
fn signal_mid_run(root: &Path, args: &[&str], signal: &str) -> (ExitStatus, bool) {
    let mut bb: Child = Command::new(env!("CARGO_BIN_EXE_bb"))
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut server: Option<u32> = None;
    assert!(
        wait_until(Duration::from_secs(120), || {
            server = server_pid(root);
            server.is_some()
        }),
        "server never started for {args:?}"
    );
    let group: u32 = server.unwrap();
    std::thread::sleep(Duration::from_millis(500));
    let sent: ExitStatus = Command::new("/bin/kill")
        .args([signal, &bb.id().to_string()])
        .status()
        .unwrap();
    assert!(sent.success());
    let mut status: Option<ExitStatus> = None;
    let exited: bool = wait_until(Duration::from_secs(5), || {
        status = bb.try_wait().unwrap();
        status.is_some()
    });
    if !exited {
        bb.kill().unwrap();
    }
    let gone: bool = wait_until(Duration::from_secs(1), || !group_alive(group));
    if !gone {
        // Clean up a red run so no orphan outlives the test.
        let _killed: std::io::Result<ExitStatus> = Command::new("/bin/kill")
            .args(["-KILL", "--", &format!("-{group}")])
            .status();
    }
    (status.unwrap_or_else(|| bb.wait().unwrap()), gone)
}

#[test]
fn fatal_signal_to_bb_mcp_ends_a_server_that_ignores_eof_and_term() {
    let root: PathBuf = stubborn_fixture();
    let project: String = root
        .join("McpServer/_McpServer.csproj")
        .display()
        .to_string();
    let cases: [(&[&str], &str, i32); 3] = [
        (&["mcp", "log", &project], "-TERM", 143),
        (&["mcp", "call", &project, "Hold", "{}"], "-TERM", 143),
        (&["mcp", "call", &project, "Hold", "{}"], "-HUP", 129),
    ];
    for (args, signal, code) in cases {
        let (status, gone): (ExitStatus, bool) = signal_mid_run(&root, args, signal);
        assert!(gone, "{args:?} {signal}: server group outlived bb");
        assert_eq!(status.code(), Some(code), "{args:?} {signal}: {status}");
    }
    std::fs::remove_dir_all(&root).unwrap();
}
