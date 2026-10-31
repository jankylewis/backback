//! Headless debugger probe through the real DAP adapter.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_dap::{DapError, DapSession, Event, Scope, SourceBreakpoint, StackFrame, Variable};
use serde_json::Value;
use tokio::process::Command;

const DEADLINE: Duration = Duration::from_secs(30);

/// Probe errors.
#[derive(Debug, thiserror::Error)]
pub enum DebugError {
    /// Invalid target kind.
    #[error("debug target must be a .csproj or .dll")]
    Target,
    /// File or process error.
    #[error("debug I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Build failed.
    #[error("dotnet build failed: {0}")]
    Build(String),
    /// Build timed out.
    #[error("dotnet command timed out")]
    Timeout,
    /// Debug adapter error.
    #[error(transparent)]
    Dap(#[from] DapError),
    /// Target did not stop at a breakpoint.
    #[error("debug session ended before the breakpoint")]
    NoStop,
}

/// Debug one target, print the first stop and its locals, then resume.
///
/// # Errors
/// Missing target, failed build, or DAP failure.
pub async fn run(target: &Path, source: &Path, line: u32) -> Result<(), DebugError> {
    let (session, mut events): (DapSession, tokio::sync::mpsc::Receiver<Event>) =
        start(target, source, line).await?;
    let mut saw_stop: bool = false;
    loop {
        match DapSession::next_event(&mut events, DEADLINE).await? {
            Event::Stopped(body) => {
                saw_stop = true;
                let thread_id: i64 = body
                    .get("threadId")
                    .and_then(Value::as_i64)
                    .ok_or(DebugError::NoStop)?;
                println!(
                    "stopped   : {}",
                    body.get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                );
                print_stack_and_locals(&session, thread_id).await?;
                session.continue_thread(thread_id).await?;
            }
            Event::Exited(body) => println!("exited    : {body}"),
            Event::Terminated(_) => {
                return if saw_stop {
                    Ok(())
                } else {
                    Err(DebugError::NoStop)
                }
            }
            Event::Output(body) => {
                if let Some(output) = body.get("output").and_then(Value::as_str) {
                    print!("{output}");
                }
            }
            Event::Breakpoint(_) | Event::Other(_, _) => {}
        }
    }
}

async fn start(
    target: &Path,
    source: &Path,
    line: u32,
) -> Result<(DapSession, tokio::sync::mpsc::Receiver<Event>), DebugError> {
    let program: PathBuf = resolve_target(target).await?;
    let source: PathBuf = source.canonicalize()?;
    let cwd: &Path = program.parent().ok_or(DebugError::Target)?;
    let mut session: DapSession = DapSession::spawn(DEADLINE)?;
    let events: tokio::sync::mpsc::Receiver<Event> =
        session.take_events().ok_or(DapError::Closed)?;
    let caps: bb_dap::Capabilities = session.initialize().await?;
    println!(
        "goto      : {}",
        if caps.supports_goto_targets_request {
            "available"
        } else {
            "hidden"
        }
    );
    session
        .launch(&program, &[], cwd, &BTreeMap::new(), false)
        .await?;
    let response: Value = session
        .set_breakpoints(
            &source,
            &[SourceBreakpoint {
                line,
                condition: None,
            }],
        )
        .await?;
    println!("breakpoint: {}:{} {}", source.display(), line, response);
    session.configuration_done().await?;
    Ok((session, events))
}

async fn print_stack_and_locals(session: &DapSession, thread_id: i64) -> Result<(), DebugError> {
    let frames: Vec<StackFrame> = session.stack_trace(thread_id).await?;
    for frame in &frames {
        println!(
            "frame     : {}:{} {}",
            frame
                .source
                .as_ref()
                .and_then(|s: &Value| s.get("path"))
                .and_then(Value::as_str)
                .unwrap_or("?"),
            frame.line,
            frame.name
        );
    }
    if let Some(frame) = frames.first() {
        let scopes: Vec<Scope> = session.scopes(frame.id).await?;
        for scope in scopes {
            let vars: Vec<Variable> = session.variables(scope.variables_reference).await?;
            for var in vars {
                println!("local     : {} = {}", var.name, var.value);
            }
        }
    }
    Ok(())
}

async fn resolve_target(target: &Path) -> Result<PathBuf, DebugError> {
    match target
        .extension()
        .and_then(|ext: &std::ffi::OsStr| ext.to_str())
    {
        Some("dll") => Ok(target.canonicalize()?),
        Some("csproj") => {
            let project: PathBuf = target.canonicalize()?;
            let output: std::process::Output = timed_command(
                Command::new("dotnet")
                    .arg("build")
                    .arg(&project)
                    .arg("-v:q"),
            )
            .await?;
            if !output.status.success() {
                return Err(DebugError::Build(
                    String::from_utf8_lossy(&output.stderr).into_owned(),
                ));
            }
            let output: std::process::Output = timed_command(
                Command::new("dotnet")
                    .arg("msbuild")
                    .arg(&project)
                    .arg("-getProperty:TargetPath"),
            )
            .await?;
            if !output.status.success() {
                return Err(DebugError::Build(
                    String::from_utf8_lossy(&output.stderr).into_owned(),
                ));
            }
            let path: PathBuf = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
            Ok(path.canonicalize()?)
        }
        _ => Err(DebugError::Target),
    }
}

async fn timed_command(command: &mut Command) -> Result<std::process::Output, DebugError> {
    tokio::time::timeout(DEADLINE, command.kill_on_drop(true).output())
        .await
        .map_err(|_: _| DebugError::Timeout)?
        .map_err(DebugError::Io)
}
