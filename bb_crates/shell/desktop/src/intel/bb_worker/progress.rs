//! The server's run log: every line goes to a file, and each line that
//! loads a new distinct project also becomes a progress event for the
//! status bar. Batch lines are never summed: a batch can reload a project.

use std::io::Write;

use bb_lsp::roslyn::LoadState;
use tokio::sync::mpsc::UnboundedSender;

use super::protocol::Event;

/// A log sink that counts loaded projects on the way to its file.
pub(crate) struct ProgressLog<W: Write> {
    inner: W,
    events: UnboundedSender<Event>,
    partial: Vec<u8>,
    state: LoadState,
}

impl<W: Write> ProgressLog<W> {
    pub(crate) fn new(inner: W, events: UnboundedSender<Event>, state: LoadState) -> Self {
        Self {
            inner,
            events,
            partial: Vec::new(),
            state,
        }
    }

    fn line(&mut self, line: &[u8]) {
        let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(line);
        if !self.state.observe(&text) {
            return;
        }
        // A closed UI only means nobody watches the load any more.
        let _sent: Result<(), tokio::sync::mpsc::error::SendError<Event>> =
            self.events.send(Event::Progress {
                loaded: self.state.loaded(),
                total: self.state.total(),
            });
    }
}

impl<W: Write> Write for ProgressLog<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written: usize = self.inner.write(buf)?;
        let taken: &[u8] = buf.get(..written).unwrap_or(buf);
        for byte in taken {
            if *byte == b'\n' {
                let line: Vec<u8> = std::mem::take(&mut self.partial);
                self.line(&line);
            } else {
                self.partial.push(*byte);
            }
        }
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bb_dotnet::ProjectPath;

    const ROOT: &str = "/repo/_Harness";
    const NAMES: [&str; 4] = [
        "Core/_Core",
        "_NUnitApex/_NUnitApex",
        "Xunit/_Xunit",
        "Mcp/_Mcp",
    ];

    fn state() -> LoadState {
        let projects: Vec<ProjectPath> = NAMES
            .iter()
            .map(|n: &&str| ProjectPath::from_path(format!("{ROOT}/{n}.csproj").into()))
            .collect();
        LoadState::new(&projects)
    }

    fn done(name: &str) -> String {
        format!("[info] Successfully completed load of {ROOT}/{name}.csproj\n")
    }

    #[test]
    fn a_reloaded_project_split_across_writes_counts_once_and_batches_never_count() {
        let (tx, mut rx): (
            UnboundedSender<Event>,
            tokio::sync::mpsc::UnboundedReceiver<Event>,
        ) = tokio::sync::mpsc::unbounded_channel();
        let mut log: ProgressLog<Vec<u8>> = ProgressLog::new(Vec::new(), tx, state());
        let first: String = done(NAMES[0]);
        let (head, tail): (&str, &str) = first.split_at(30);
        log.write_all(head.as_bytes()).unwrap();
        log.write_all(tail.as_bytes()).unwrap();
        log.write_all(b"[info] Completed (re)load of 1 project(s) in 1s\n[info] other\n")
            .unwrap();
        for name in [NAMES[0], NAMES[1], NAMES[2]] {
            log.write_all(done(name).as_bytes()).unwrap();
        }
        log.write_all(b"[info] Completed (re)load of 9 project(s)\n")
            .unwrap();
        let mut seen: Vec<(usize, usize)> = Vec::new();
        while let Ok(Event::Progress { loaded, total }) = rx.try_recv() {
            seen.push((loaded, total));
        }
        assert_eq!(seen, vec![(1, 4), (2, 4), (3, 4)]);
        assert!(String::from_utf8(log.inner)
            .unwrap()
            .contains("[info] other"));
    }
}
