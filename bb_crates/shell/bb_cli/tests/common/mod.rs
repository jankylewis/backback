//! Run the real `bb` binary with a deadline and capture what it printed.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// No `bb` call in these tests may take longer than this.
const DEADLINE: Duration = Duration::from_secs(30);

/// What one `bb` run produced.
pub struct Run {
    /// Exit code (`None` when killed by a signal).
    pub code: Option<i32>,
    /// Standard output.
    pub out: String,
    /// Standard error.
    pub err: String,
}

impl Run {
    /// Both streams, for "never printed anywhere" checks.
    pub fn all(&self) -> String {
        format!("{}{}", self.out, self.err)
    }
}

/// The `_Harness` fixture solution folder.
pub fn harness() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/_Harness")
}

/// Run `bb` with `args`; kill it and fail the test after [`DEADLINE`].
pub fn bb(args: &[&str]) -> Run {
    let mut child: Child = Command::new(env!("CARGO_BIN_EXE_bb"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let started: Instant = Instant::now();
    let status: ExitStatus = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if started.elapsed() > DEADLINE {
            child.kill().unwrap();
            panic!("bb {args:?} ran past {DEADLINE:?}");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    Run {
        code: status.code(),
        out: drain(child.stdout.take()),
        err: drain(child.stderr.take()),
    }
}

fn drain(pipe: Option<impl Read>) -> String {
    let mut text: String = String::new();
    pipe.unwrap().read_to_string(&mut text).unwrap();
    text
}
