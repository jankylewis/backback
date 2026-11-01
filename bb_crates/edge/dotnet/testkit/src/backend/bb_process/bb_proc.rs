//! One child process with a deadline: the only way the engine runs tools.

use std::ffi::OsString;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::time::{Duration, Instant};

use bb_process::OwnedGroup;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;

use super::StopFlag;
use crate::TestkitError;

/// Lines of error output kept in a [`TestkitError::Failed`].
const DETAIL_LINES: usize = 12;

/// A program, its arguments, its folder and extra environment.
#[derive(Debug, Clone)]
pub struct Call {
    /// The program.
    pub program: PathBuf,
    /// The arguments.
    pub args: Vec<OsString>,
    /// The working folder.
    pub cwd: PathBuf,
    /// Extra environment variables.
    pub env: Vec<(String, String)>,
    stop: Option<StopFlag>,
}

/// What a finished process left.
#[derive(Debug, Clone)]
pub struct Finished {
    /// The exit code (`None`: killed by a signal).
    pub code: Option<i32>,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Wall time.
    pub elapsed: Duration,
}

impl Call {
    /// A call of `program` in `cwd`.
    pub fn new(program: impl Into<PathBuf>, cwd: &Path) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: cwd.to_path_buf(),
            env: Vec::new(),
            stop: None,
        }
    }

    /// Add arguments.
    #[must_use]
    pub fn args<I: IntoIterator<Item = S>, S: Into<OsString>>(mut self, args: I) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    #[must_use]
    pub fn stoppable(mut self, stop: StopFlag) -> Self {
        self.stop = Some(stop);
        self
    }

    /// The command, ready to spawn; it dies with its handle.
    pub fn command(&self) -> Command {
        let mut cmd: Command = Command::new(&self.program);
        cmd.args(&self.args)
            .current_dir(&self.cwd)
            .envs(self.env.iter().map(|(k, v): &(String, String)| (k, v)))
            .stdin(Stdio::null())
            .process_group(0)
            .kill_on_drop(true);
        cmd
    }

    /// The program's file name, for messages.
    pub fn name(&self) -> String {
        let program: String = self.program.file_name().map_or_else(
            || self.program.display().to_string(),
            |n: &std::ffi::OsStr| n.to_string_lossy().into_owned(),
        );
        let first: String = self
            .args
            .iter()
            .take(2)
            .map(|a: &OsString| a.to_string_lossy().into_owned())
            .collect::<Vec<String>>()
            .join(" ");
        format!("{program} {first}")
    }

    /// Run to the end or until `deadline`, whatever the exit code. Past
    /// the deadline the whole process tree is killed, not only the program.
    ///
    /// # Errors
    /// The program cannot start, or the deadline passes.
    pub async fn finish(&self, deadline: Duration) -> Result<Finished, TestkitError> {
        let started: Instant = Instant::now();
        let mut child: Child = self
            .command()
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source: std::io::Error| self.spawn_error(source))?;
        let mut group: Option<OwnedGroup> = child
            .id()
            .and_then(NonZeroU32::new)
            .map(OwnedGroup::register);
        let stdout: JoinHandle<Vec<u8>> = tokio::spawn(read_all(child.stdout.take()));
        let stderr: JoinHandle<Vec<u8>> = tokio::spawn(read_all(child.stderr.take()));
        let status: ExitStatus = tokio::select! {
            waited = child.wait() => waited.map_err(|source: std::io::Error| self.spawn_error(source))?,
            () = tokio::time::sleep(deadline) => {
                drop(group.take());
                return Err(TestkitError::Timeout { what: self.name(), after: deadline });
            }
            () = wait_stop(self.stop.as_ref()) => {
                drop(group.take());
                return Err(TestkitError::Cancelled);
            }
        };
        drop(group.take());
        Ok(Finished {
            code: status.code(),
            stdout: String::from_utf8_lossy(&stdout.await.unwrap_or_default()).into_owned(),
            stderr: String::from_utf8_lossy(&stderr.await.unwrap_or_default()).into_owned(),
            elapsed: started.elapsed(),
        })
    }

    fn spawn_error(&self, source: std::io::Error) -> TestkitError {
        TestkitError::Spawn {
            program: self.program.display().to_string(),
            source,
        }
    }

    /// Run and require exit code 0.
    ///
    /// # Errors
    /// See [`Call::finish`]; [`TestkitError::Failed`] on a non-zero exit.
    pub async fn succeed(&self, deadline: Duration) -> Result<Finished, TestkitError> {
        let done: Finished = self.finish(deadline).await?;
        if done.code == Some(0) {
            return Ok(done);
        }
        let source: &str = if done.stderr.trim().is_empty() {
            &done.stdout
        } else {
            &done.stderr
        };
        Err(self.failed(done.code, source))
    }

    /// A discovery step: exit code 0 and no exception in the error output.
    /// `dotnet vstest` and the Apex CLI both exit 0 with an empty list when
    /// the test host crashes; the exception on stderr is the only sign.
    ///
    /// # Errors
    /// See [`Call::succeed`]; [`TestkitError::Failed`] on an exception line.
    pub async fn discover(&self, deadline: Duration) -> Result<Finished, TestkitError> {
        let done: Finished = self.succeed(deadline).await?;
        if done.stderr.lines().any(|l: &str| l.contains("Exception:")) {
            return Err(self.failed(done.code, &done.stderr));
        }
        Ok(done)
    }

    fn failed(&self, code: Option<i32>, output: &str) -> TestkitError {
        let lines: Vec<&str> = output.lines().collect();
        let tail: &[&str] = lines
            .get(lines.len().saturating_sub(DETAIL_LINES)..)
            .unwrap_or_default();
        TestkitError::Failed {
            what: self.name(),
            code,
            detail: tail.join("\n"),
        }
    }
}

async fn wait_stop(stop: Option<&StopFlag>) {
    match stop {
        Some(flag) => flag.wait().await,
        None => std::future::pending::<()>().await,
    }
}

async fn read_all<R: AsyncRead + Unpin>(pipe: Option<R>) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    if let Some(mut pipe) = pipe {
        // A pipe that breaks keeps what was read so far; the exit code tells the rest.
        let _read: std::io::Result<usize> = pipe.read_to_end(&mut bytes).await;
    }
    bytes
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
