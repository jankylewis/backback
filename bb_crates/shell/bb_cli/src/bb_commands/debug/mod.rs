//! Headless debugger probe through the real DAP adapter.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

mod bb_exit;

use bb_dap::{
    DapError, DapSession, Event, ExitSource, Scope, SourceBreakpoint, StackFrame, Variable,
};
use bb_exit::launched_exit_line;
use serde_json::Value;

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

impl From<bb_dap::target::TargetError> for DebugError {
    fn from(error: bb_dap::target::TargetError) -> Self {
        match error {
            bb_dap::target::TargetError::Kind => Self::Target,
            bb_dap::target::TargetError::Io(io) => Self::Io(io),
            bb_dap::target::TargetError::Build(text) => Self::Build(text),
            bb_dap::target::TargetError::Timeout => Self::Timeout,
        }
    }
}

/// Debug one target, print the first stop and its locals, then resume.
///
/// # Errors
/// Missing target, failed build, or DAP failure.
pub async fn run(target: &Path, source: &Path, line: u32) -> Result<(), DebugError> {
    let (session, mut events, exit_source): (
        DapSession,
        tokio::sync::mpsc::Receiver<Event>,
        ExitSource,
    ) = start(target, source, line).await?;
    let mut saw_stop: bool = false;
    loop {
        let event: Event = DapSession::next_event(&mut events, DEADLINE).await?;
        let exit_code: Option<i64> = event.exit_code();
        let exit_signal: Option<i64> = event.exit_signal();
        match event {
            Event::Stopped(body) => {
                saw_stop = true;
                println!(
                    "stopped   : {}",
                    body.get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                );
                let thread_id: i64 = thread_of(&body)?;
                print_stack_and_locals(&session, thread_id).await?;
                session.continue_thread(thread_id).await?;
            }
            Event::ConditionError { message, stopped } => {
                println!("condition : {message}");
                session.continue_thread(thread_of(&stopped)?).await?;
            }
            Event::Exited(_) => {
                println!(
                    "exited    : {}",
                    launched_exit_line(exit_code, exit_signal, exit_source)
                );
            }
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

fn thread_of(stopped: &Value) -> Result<i64, DebugError> {
    stopped
        .get("threadId")
        .and_then(Value::as_i64)
        .ok_or(DebugError::NoStop)
}

async fn start(
    target: &Path,
    source: &Path,
    line: u32,
) -> Result<(DapSession, tokio::sync::mpsc::Receiver<Event>, ExitSource), DebugError> {
    let program: PathBuf = bb_dap::target::resolve(target, DEADLINE).await?;
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
    let exit_source: ExitSource = caps.launch_exit_source();
    println!(
        "exit code : {}",
        match exit_source {
            ExitSource::ProcessStatus => "process status",
            ExitSource::AdapterWord => "adapter word",
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
    Ok((session, events, exit_source))
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
