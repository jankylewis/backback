//! The measured items. Each returns one [`Row`].

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use bb_dotnet::{ProjectInfo, SolutionPath};
use bb_syntax::{CSharpSyntax, OutlineItem, Tree};

use super::stats::{repeat, timed, Summary};
use super::{BenchError, Row, Unit};
use crate::commands::{read, scan, CommandError};

/// Deadline for one `bb doctor` run.
const COLD_START_TIMEOUT: Duration = Duration::from_secs(30);
/// Size of the generated outline input.
const GENERATED_BYTES: usize = 50 * 1024;
/// Folders that never hold source.
const SKIP_DIRS: [&str; 3] = ["bin", "obj", ".git"];

/// Wall time of a fresh `bb doctor` process.
pub(crate) fn cold_start() -> Result<Row, CommandError> {
    let exe: PathBuf = current_exe()?;
    let summary: Summary = timed(|| -> Result<(), CommandError> { doctor(&exe) })?;
    Ok(Row::new(
        "cold-start",
        Unit::Micros,
        summary,
        "bb doctor".to_owned(),
    ))
}

fn doctor(exe: &Path) -> Result<(), CommandError> {
    let mut child: Child = Command::new(exe)
        .arg("doctor")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|source: std::io::Error| io("start bb doctor", source))?;
    let started: Instant = Instant::now();
    loop {
        let status: Option<ExitStatus> = child
            .try_wait()
            .map_err(|source: std::io::Error| io("wait for bb doctor", source))?;
        match status {
            Some(s) if s.success() => return Ok(()),
            Some(s) => return Err(BenchError::ColdStartFailed(s).into()),
            None if started.elapsed() > COLD_START_TIMEOUT => {
                let _killed: std::io::Result<()> = child.kill();
                return Err(BenchError::ColdStartTimeout(COLD_START_TIMEOUT).into());
            }
            None => std::thread::sleep(Duration::from_millis(1)),
        }
    }
}

/// Outline parse of the largest fixture `.cs` and of a generated 50 kB file.
pub(crate) fn outlines(fixtures: &Path) -> Result<[Row; 2], CommandError> {
    let largest: PathBuf = largest_cs(fixtures)?.ok_or(BenchError::NoCsFile)?;
    let text: String = read(&largest)?;
    let name: String = largest
        .file_name()
        .map(|n: &std::ffi::OsStr| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let fixture: Row = outline_row("outline-largest", &text, &name)?;
    let generated: Row = outline_row("outline-50k", &generated_cs(), "generated")?;
    Ok([fixture, generated])
}

fn outline_row(item: &'static str, text: &str, name: &str) -> Result<Row, CommandError> {
    let mut syntax: CSharpSyntax = CSharpSyntax::new()?;
    let mut items: usize = 0;
    let summary: Summary = timed(|| -> Result<(), CommandError> {
        let tree: Tree = syntax.parse(text, None)?;
        let outline: Vec<OutlineItem> = CSharpSyntax::outline(&tree, text);
        items = std::hint::black_box(outline).len();
        Ok(())
    })?;
    let detail: String = format!("{name} {} B, {items} items", text.len());
    Ok(Row::new(item, Unit::Micros, summary, detail))
}

/// A class with enough small methods to pass [`GENERATED_BYTES`].
pub(crate) fn generated_cs() -> String {
    let mut text: String =
        String::from("namespace Bench.Generated;\n\npublic class Generated\n{\n");
    let mut index: usize = 0;
    while text.len() < GENERATED_BYTES {
        let _infallible: std::fmt::Result = write!(
            text,
            "    public int Method{index}(int value)\n    {{\n        int doubled = value * 2;\n        return doubled + {index};\n    }}\n\n"
        );
        index = index.saturating_add(1);
    }
    text.push_str("}\n");
    text
}

fn largest_cs(dir: &Path) -> Result<Option<PathBuf>, CommandError> {
    let mut best: Option<(u64, PathBuf)> = None;
    let entries: std::fs::ReadDir =
        std::fs::read_dir(dir).map_err(|source: std::io::Error| io("list fixtures", source))?;
    for entry in entries {
        let path: PathBuf = entry
            .map_err(|source: std::io::Error| io("list fixtures", source))?
            .path();
        let name: String = path
            .file_name()
            .map(|n: &std::ffi::OsStr| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let found: Option<PathBuf> = if path.is_dir() {
            if SKIP_DIRS.contains(&name.as_str()) {
                None
            } else {
                largest_cs(&path)?
            }
        } else {
            path.extension()
                .is_some_and(|e: &std::ffi::OsStr| e == "cs")
                .then_some(path)
        };
        if let Some(candidate) = found {
            let size: u64 = std::fs::metadata(&candidate)
                .map_err(|source: std::io::Error| io("size a fixture", source))?
                .len();
            if best
                .as_ref()
                .is_none_or(|(b, _): &(u64, PathBuf)| size > *b)
            {
                best = Some((size, candidate));
            }
        }
    }
    Ok(best.map(|(_, p): (u64, PathBuf)| p))
}

/// Solution listing plus every project file, as `bb scan` reads them.
pub(crate) fn scan(item: &'static str, solution: &Path) -> Result<Row, CommandError> {
    let sln: SolutionPath = SolutionPath::new(solution.to_path_buf())?;
    let mut projects: usize = 0;
    let summary: Summary = timed(|| -> Result<(), CommandError> {
        let infos: Vec<ProjectInfo> = scan::load(&sln)?;
        projects = infos.len();
        Ok(())
    })?;
    let name: String = solution
        .file_name()
        .map(|n: &std::ffi::OsStr| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(Row::new(
        item,
        Unit::Micros,
        summary,
        format!("{name}, {projects} projects"),
    ))
}

/// `bb audit` of the whole repo.
pub(crate) fn audit(repo: &Path) -> Result<Row, CommandError> {
    let mut found: usize = 0;
    let summary: Summary = timed(|| -> Result<(), CommandError> {
        found = bb_audit::audit(repo)?.len();
        Ok(())
    })?;
    Ok(Row::new(
        "audit-repo",
        Unit::Micros,
        summary,
        format!("{found} violations"),
    ))
}

/// Size of the running `bb` binary (build with `--release` for the baseline).
pub(crate) fn binary_size() -> Result<Row, CommandError> {
    let exe: PathBuf = current_exe()?;
    let summary: Summary = repeat(|| -> Result<u64, CommandError> {
        Ok(std::fs::metadata(&exe)
            .map_err(|source: std::io::Error| io("size the bb binary", source))?
            .len())
    })?;
    let profile: &str = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    Ok(Row::new(
        "binary-size",
        Unit::Bytes,
        summary,
        format!("{profile} build"),
    ))
}

fn current_exe() -> Result<PathBuf, CommandError> {
    std::env::current_exe().map_err(|source: std::io::Error| io("find the bb binary", source))
}

fn io(what: &'static str, source: std::io::Error) -> CommandError {
    BenchError::Io { what, source }.into()
}
