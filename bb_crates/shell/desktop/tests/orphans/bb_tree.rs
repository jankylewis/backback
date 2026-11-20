//! A real `backback` on a C# + Rust + Python folder, and the process tree it
//! grows, read with `ps` (BBEOD42).

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The servers a window on the fixture starts, as `ps` names them.
pub const ROSLYN: &str = "Microsoft.CodeAnalysis.LanguageServer";
pub const RUST: &str = "rust-analyzer";
pub const PYRIGHT: &str = "pyright-langserver";

/// One `ps` row.
#[derive(Clone, Debug)]
pub struct Row {
    pub pid: u32,
    pub ppid: u32,
    pub pgid: u32,
    pub zombie: bool,
    pub command: String,
}

pub fn rows() -> Vec<Row> {
    let out: std::process::Output = Command::new("/bin/ps")
        .args(["-A", "-ww", "-o", "pid=,ppid=,pgid=,stat=,command="])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line: &str| {
            let mut words: std::str::SplitWhitespace<'_> = line.split_whitespace();
            let id: u32 = words.next()?.parse().ok()?;
            let parent: u32 = words.next()?.parse().ok()?;
            let group: u32 = words.next()?.parse().ok()?;
            let stat: &str = words.next()?;
            let command: String = words.collect::<Vec<&str>>().join(" ");
            Some(Row {
                pid: id,
                ppid: parent,
                pgid: group,
                zombie: stat.starts_with('Z'),
                command,
            })
        })
        .collect()
}

/// Every live process under `root` (by parent) or in a group one of them
/// leads, plus those already in `seen` that still run.
pub fn tree(root: u32, seen: &BTreeMap<u32, Row>) -> BTreeMap<u32, Row> {
    let all: Vec<Row> = rows();
    let own: u32 = all
        .iter()
        .find(|row: &&Row| row.pid == std::process::id())
        .map_or(0, |row: &Row| row.pgid);
    let mut pids: BTreeSet<u32> = BTreeSet::from([root]);
    let mut groups: BTreeSet<u32> = seen
        .values()
        .map(|row: &Row| row.pgid)
        .filter(|group: &u32| *group != own)
        .collect();
    loop {
        let before: usize = pids.len() + groups.len();
        for row in &all {
            if row.pid != root && (pids.contains(&row.ppid) || groups.contains(&row.pgid)) {
                pids.insert(row.pid);
                if row.pgid != own {
                    groups.insert(row.pgid);
                }
            }
        }
        if pids.len() + groups.len() == before {
            break;
        }
    }
    all.into_iter()
        .filter(|row: &Row| !row.zombie && (pids.contains(&row.pid) || seen.contains_key(&row.pid)))
        .filter(|row: &Row| row.pid != std::process::id())
        .map(|row: Row| (row.pid, row))
        .collect()
}

pub fn shown(rows: &BTreeMap<u32, Row>) -> Vec<String> {
    rows.values()
        .map(|row: &Row| {
            let short: String = row.command.chars().take(140).collect();
            format!("{} pgid={} {short}", row.pid, row.pgid)
        })
        .collect()
}

/// The servers this machine has; all three when `BACKBACK_REQUIRE_TOOLS`.
pub fn wanted() -> Vec<&'static str> {
    let required: bool = std::env::var_os("BACKBACK_REQUIRE_TOOLS").is_some();
    let home: PathBuf = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let mut servers: Vec<&'static str> = Vec::new();
    if required || home.join(".dotnet/tools/roslyn-language-server").is_file() {
        servers.push(ROSLYN);
    }
    if required || on_path(RUST) {
        servers.push(RUST);
    }
    if required || on_path(PYRIGHT) {
        servers.push(PYRIGHT);
    }
    servers
}

fn on_path(name: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|path: std::ffi::OsString| {
        std::env::split_paths(&path).any(|dir: PathBuf| dir.join(name).is_file())
    })
}

/// The folder (C#, Rust, Python at one root) and its state folder with
/// all three languages picked; the file to open.
pub fn fixture(base: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = base.join("app");
    let files: [(&str, &str); 7] = [
        ("App.slnx", "<Solution>\n  <Project Path=\"App.csproj\" />\n</Solution>\n"),
        (
            "App.csproj",
            "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>\n",
        ),
        (
            "Program.cs",
            "class Program { static void Main() { System.Threading.Thread.Sleep(-1); } }\n",
        ),
        ("Cargo.toml", "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        ("src/main.rs", "fn main() {}\n"),
        ("pyproject.toml", "[project]\nname = \"app\"\nversion = \"0.1.0\"\n"),
        ("tool/main.py", "def main():\n    return 1\n"),
    ];
    for (file, text) in files {
        let path: PathBuf = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    let root: PathBuf = root.canonicalize().unwrap();
    let state: PathBuf = base.join("state");
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(
        state.join("langs"),
        format!(
            "backback langs v2\npicked\tcsharp,rust,python\t{}\n",
            root.display()
        ),
    )
    .unwrap();
    (root.join("Program.cs"), root, state)
}

/// A running `backback` and what it printed so far.
pub struct App {
    pub child: Child,
    pub go: PathBuf,
    pub said: Arc<Mutex<Vec<String>>>,
}

/// A script that ends backback by itself (`crash:`, or the quit after the
/// last step) must first hold at `wait:go`, so it ends only once the test
/// saw the processes it waits for (BB-GAT-020).
pub fn launch(file: &Path, state: &Path, keys: &str, quit: bool) -> App {
    let ends_itself: bool = quit || keys.contains("crash:");
    assert!(
        !ends_itself || keys.split_whitespace().any(|step: &str| step == "wait:go"),
        "{keys:?} ends backback on its own timer: hold it at wait:go"
    );
    let go: PathBuf = state.join("go");
    let mut command: Command = Command::new(env!("CARGO_BIN_EXE_backback"));
    command
        .arg(file)
        .env("BACKBACK_STATE_DIR", state)
        .env("BB_KEYS", keys)
        .env_remove("BB_KEYS_QUIT")
        .env("BB_KEYS_GO", &go)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if quit {
        command.env("BB_KEYS_QUIT", "1");
    }
    let mut child: Child = command.spawn().unwrap();
    let stderr: ChildStderr = child.stderr.take().unwrap();
    let said: Arc<Mutex<Vec<String>>> = Arc::default();
    let into: Arc<Mutex<Vec<String>>> = Arc::clone(&said);
    let _reader: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            into.lock().unwrap().push(line);
        }
    });
    App { child, go, said }
}

impl Drop for App {
    fn drop(&mut self) {
        let _stopped: std::io::Result<()> = self.child.kill();
        let _reaped: std::io::Result<std::process::ExitStatus> = self.child.wait();
    }
}

impl App {
    /// Let a script held at `wait:go` go on.
    pub fn go(&self) {
        std::fs::create_dir_all(self.go.parent().unwrap()).unwrap();
        std::fs::write(&self.go, "go\n").unwrap();
    }

    pub fn printed(&self, wanted: &str, within: Duration) -> bool {
        let started: Instant = Instant::now();
        loop {
            if self
                .said
                .lock()
                .unwrap()
                .iter()
                .any(|l: &String| l.contains(wanted))
            {
                return true;
            }
            if started.elapsed() >= within {
                return false;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn ended_within(&mut self, within: Duration) -> bool {
        let started: Instant = Instant::now();
        while started.elapsed() < within {
            if self.child.try_wait().unwrap().is_some() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        false
    }
}
