//! The server's run log: every line goes to a file, and each project batch
//! line also becomes a progress event for the status bar.

use std::io::Write;

use tokio::sync::mpsc::UnboundedSender;

use super::protocol::Event;

/// A log sink that counts loaded projects on the way to its file.
pub(crate) struct ProgressLog<W: Write> {
    inner: W,
    events: UnboundedSender<Event>,
    partial: Vec<u8>,
    loaded: usize,
    total: usize,
}

impl<W: Write> ProgressLog<W> {
    pub(crate) fn new(inner: W, events: UnboundedSender<Event>, total: usize) -> Self {
        Self {
            inner,
            events,
            partial: Vec::new(),
            loaded: 0,
            total,
        }
    }

    fn line(&mut self, line: &[u8]) {
        let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(line);
        let batch: usize = bb_lsp::roslyn::batch_projects(&text);
        if batch == 0 {
            return;
        }
        self.loaded = self.loaded.saturating_add(batch).min(self.total);
        // A closed UI only means nobody watches the load any more.
        let _sent: Result<(), tokio::sync::mpsc::error::SendError<Event>> =
            self.events.send(Event::Progress {
                loaded: self.loaded,
                total: self.total,
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

    #[test]
    fn batch_lines_split_across_writes_add_up_to_the_total() {
        let (tx, mut rx): (
            UnboundedSender<Event>,
            tokio::sync::mpsc::UnboundedReceiver<Event>,
        ) = tokio::sync::mpsc::unbounded_channel();
        let mut log: ProgressLog<Vec<u8>> = ProgressLog::new(Vec::new(), tx, 4);
        log.write_all(b"[info] Completed (re)load of 1 pro")
            .unwrap();
        log.write_all(b"ject(s) in 0.4s\n[info] other\n").unwrap();
        log.write_all(b"[info] Completed (re)load of 3 project(s) in 1s\n")
            .unwrap();
        log.write_all(b"[info] Completed (re)load of 9 project(s)\n")
            .unwrap();
        let mut seen: Vec<(usize, usize)> = Vec::new();
        while let Ok(Event::Progress { loaded, total }) = rx.try_recv() {
            seen.push((loaded, total));
        }
        assert_eq!(seen, vec![(1, 4), (4, 4), (4, 4)]);
        assert!(String::from_utf8(log.inner)
            .unwrap()
            .contains("[info] other"));
    }
}
