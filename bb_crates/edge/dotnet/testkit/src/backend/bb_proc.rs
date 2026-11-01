//! One child process with a deadline: the only way the engine runs tools.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::time::{Duration, Instant};

use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;
use tokio::time::error::Elapsed;

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
        }
    }

    /// Add arguments.
    #[must_use]
    pub fn args<I: IntoIterator<Item = S>, S: Into<OsString>>(mut self, args: I) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// The command, ready to spawn; it dies with its handle.
    pub fn command(&self) -> Command {
        let mut cmd: Command = Command::new(&self.program);
        cmd.args(&self.args)
            .current_dir(&self.cwd)
            .envs(self.env.iter().map(|(k, v): &(String, String)| (k, v)))
            .stdin(Stdio::null())
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
        let stdout: JoinHandle<Vec<u8>> = tokio::spawn(read_all(child.stdout.take()));
        let stderr: JoinHandle<Vec<u8>> = tokio::spawn(read_all(child.stderr.take()));
        let Ok(waited): Result<std::io::Result<ExitStatus>, Elapsed> =
            tokio::time::timeout(deadline, child.wait()).await
        else {
            super::tree::stop_tree(&mut child).await;
            return Err(TestkitError::Timeout {
                what: self.name(),
                after: deadline,
            });
        };
        let status: ExitStatus =
            waited.map_err(|source: std::io::Error| self.spawn_error(source))?;
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

async fn read_all<R: AsyncRead + Unpin>(pipe: Option<R>) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    if let Some(mut pipe) = pipe {
        // A pipe that breaks keeps what was read so far; the exit code tells the rest.
        let _read: std::io::Result<usize> = pipe.read_to_end(&mut bytes).await;
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_timed_out_call_leaves_none_of_its_processes_behind() {
        let pids: PathBuf =
            std::env::temp_dir().join(format!("bb-proc-tree-{}.txt", std::process::id()));
        let _fresh: std::io::Result<()> = std::fs::remove_file(&pids);
        let script: String = format!(
            "sleep 30 & echo $! >> {0}; sleep 30 & echo $! >> {0}; wait",
            pids.display()
        );
        let call: Call = Call::new("sh", &std::env::temp_dir()).args(["-c", script.as_str()]);
        let outcome: Result<Finished, TestkitError> = call.finish(Duration::from_millis(500)).await;
        tokio::time::sleep(Duration::from_millis(300)).await;
        let written: String = std::fs::read_to_string(&pids).unwrap();
        let _gone: std::io::Result<()> = std::fs::remove_file(&pids);
        let children: Vec<&str> = written.lines().collect();
        let alive: Vec<&&str> = children
            .iter()
            .filter(|pid: &&&str| {
                std::process::Command::new("kill")
                    .args(["-0", pid])
                    .status()
                    .unwrap()
                    .success()
            })
            .collect();
        assert!(
            matches!(outcome, Err(TestkitError::Timeout { .. })),
            "{outcome:?}"
        );
        assert_eq!(children.len(), 2, "{written}");
        assert!(alive.is_empty(), "children left running: {alive:?}");
    }
}
