//! `bb probe`: load the solution of a file through Roslyn (restore off), ask
//! one feature at one position, print the answer and how long it took.

mod bb_print;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bb_intel::{
    ActionItem, CompletionEntry, Config, Diagnostic, DocPath, FileChange, HoverInfo, IntelError,
    Location, Session, Span,
};
use bb_lsp::roslyn::Restore;
use bb_text::{Buffer, CharIdx, LspPoint};

use super::{bb_open, CommandError};
use crate::cli::{Feature, Probe};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const LOAD_TIMEOUT: Duration = Duration::from_secs(300);

/// `bb probe` errors.
#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    /// No `.sln`/`.slnx` in the file's folder or above it.
    #[error("no .sln or .slnx above {0}; pass --sln")]
    NoSolution(PathBuf),
    /// A folder holds more than one solution.
    #[error("{} solutions in {}; pass --sln", .1.len(), .0.display())]
    Ambiguous(PathBuf, Vec<PathBuf>),
    /// The position is not inside the file.
    #[error("line {0} column {1} is not in the file")]
    OutOfFile(u32, u32),
    /// A session call failed.
    #[error(transparent)]
    Intel(#[from] IntelError),
}

/// What a feature answered.
pub(crate) enum Answer {
    Completion(Vec<CompletionEntry>),
    Hover(Option<HoverInfo>),
    Locations(Vec<Location>),
    Rename(Vec<FileChange>),
    Actions(Vec<ActionItem>),
    Diagnostics(Vec<Diagnostic>),
}

pub(crate) async fn run(probe: &Probe) -> Result<(), CommandError> {
    let file: PathBuf = canonical(&probe.file)?;
    let at: LspPoint = to_point(&super::read(&file)?, probe.at)?;
    let solution: PathBuf = match &probe.solution {
        Some(given) => canonical(given)?,
        None => find_solution(&file)?,
    };
    let root: PathBuf = solution.parent().map(Path::to_path_buf).unwrap_or_default();
    let log_dir: PathBuf = std::env::temp_dir().join("backback-roslyn-logs");
    std::fs::create_dir_all(&log_dir).map_err(|source: std::io::Error| CommandError::Read {
        path: log_dir.clone(),
        source,
    })?;
    let (log_path, log): (PathBuf, std::fs::File) = bb_open::create_log(&log_dir)?;
    let config: Config = Config {
        scratch: bb_open::scratch_dir(&solution, std::process::id()),
        solution,
        restore: Restore::Off,
        log_dir,
        request_timeout: REQUEST_TIMEOUT,
        load_timeout: LOAD_TIMEOUT,
    };
    println!("solution  : {}", config.solution.display());
    println!("log       : {}", log_path.display());
    let started: Instant = Instant::now();
    let mut session: Session = Session::start(&config, log)
        .await
        .map_err(ProbeError::from)?;
    println!(
        "loaded    : {:?} (start {:?})",
        session.loaded_in(),
        started.elapsed()
    );
    let asked: Result<(), ProbeError> =
        ask_and_print(&mut session, &file, (probe, at), &root).await;
    let stopped: Result<(), IntelError> = session.shutdown().await;
    asked?;
    stopped.map_err(ProbeError::from)?;
    Ok(())
}

async fn ask_and_print(
    session: &mut Session,
    file: &Path,
    (probe, at): (&Probe, LspPoint),
    root: &Path,
) -> Result<(), ProbeError> {
    let doc: DocPath = DocPath::new(file.to_path_buf())?;
    let opened: Instant = Instant::now();
    let found: Vec<Diagnostic> = session.open(&doc).await?;
    println!(
        "open      : {:?} ({} diagnostics)",
        opened.elapsed(),
        found.len()
    );
    println!(
        "at        : {}:{} (lsp {}:{})",
        probe.at.0, probe.at.1, at.line, at.character
    );
    let mut samples: Vec<Duration> = Vec::with_capacity(probe.repeat);
    let mut answer: Option<Answer> = None;
    for _ in 0..probe.repeat {
        let one: Instant = Instant::now();
        answer = Some(ask(session, &doc, at, &probe.feature).await?);
        samples.push(one.elapsed());
    }
    bb_print::timings(&samples);
    if let Some(answer) = answer {
        bb_print::answer(&answer, root);
    }
    Ok(())
}

async fn ask(
    session: &Session,
    doc: &DocPath,
    at: LspPoint,
    feature: &Feature,
) -> Result<Answer, IntelError> {
    let t: Duration = REQUEST_TIMEOUT;
    Ok(match feature {
        Feature::Completion => Answer::Completion(session.completion(doc, at, t).await?),
        Feature::Hover => Answer::Hover(session.hover(doc, at, t).await?),
        Feature::Definition => Answer::Locations(session.definition(doc, at, t).await?),
        Feature::References => Answer::Locations(session.references(doc, at, t).await?),
        Feature::Rename(name) => Answer::Rename(session.rename(doc, at, name, t).await?),
        Feature::Actions => {
            let span: Span = Span { start: at, end: at };
            Answer::Actions(session.code_actions(doc, span, t).await?)
        }
        Feature::Diagnostics => Answer::Diagnostics(session.diagnostics(doc, t).await?),
    })
}

/// A 1-based line and 1-based char column, as an LSP point.
fn to_point(text: &str, (line, col): (u32, u32)) -> Result<LspPoint, ProbeError> {
    line_char(&Buffer::from_text(text), line, col).ok_or(ProbeError::OutOfFile(line, col))
}

fn line_char(buffer: &Buffer, line: u32, col: u32) -> Option<LspPoint> {
    let row: u32 = line.checked_sub(1)?;
    let start: CharIdx = buffer
        .from_lsp(LspPoint {
            line: row,
            character: 0,
        })
        .ok()?;
    let end: CharIdx = buffer
        .from_lsp(LspPoint {
            line: row,
            character: u32::MAX,
        })
        .ok()?;
    let offset: usize = usize::try_from(col.checked_sub(1)?).ok()?;
    let target: usize = start.raw().checked_add(offset)?;
    if target > end.raw() {
        return None;
    }
    buffer.to_lsp(CharIdx::new(target)).ok()
}

/// The nearest folder at or above `file` with exactly one solution.
fn find_solution(file: &Path) -> Result<PathBuf, ProbeError> {
    for dir in file.ancestors().skip(1) {
        let Ok(entries): std::io::Result<std::fs::ReadDir> = std::fs::read_dir(dir) else {
            continue;
        };
        let mut found: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|e: std::fs::DirEntry| e.path())
            .filter(|p: &PathBuf| {
                p.extension()
                    .is_some_and(|x: &std::ffi::OsStr| x == "sln" || x == "slnx")
            })
            .collect();
        found.sort();
        if found.len() > 1 {
            return Err(ProbeError::Ambiguous(dir.to_path_buf(), found));
        }
        if let Some(one) = found.pop() {
            return Ok(one);
        }
    }
    Err(ProbeError::NoSolution(file.to_path_buf()))
}

fn canonical(path: &Path) -> Result<PathBuf, CommandError> {
    std::fs::canonicalize(path).map_err(|source: std::io::Error| CommandError::Read {
        path: path.to_path_buf(),
        source,
    })
}
