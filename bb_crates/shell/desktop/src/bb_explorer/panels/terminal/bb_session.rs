//! One shell under a PTY (BBEOD11). `alacritty_terminal` reads and parses the
//! output on its own thread; the panel takes a snapshot of the visible rows.
//! Start it on the UI thread: a shell spawned from a background-QoS thread
//! would be throttled (BB-PRC-002).

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use alacritty_terminal::event::{Event, EventListener, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, EventLoopSender, Msg};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::tty;
use tokio::sync::mpsc::UnboundedSender;

/// Lines kept above the screen; small, so a terminal stays light.
const SCROLLBACK: usize = 2_000;

/// Wakes the panel when output arrives; answers the shell's queries.
#[derive(Clone)]
pub(crate) struct Proxy {
    wake: UnboundedSender<()>,
    pty: Arc<OnceLock<EventLoopSender>>,
}

impl EventListener for Proxy {
    fn send_event(&self, event: Event) {
        if let Event::PtyWrite(text) = event {
            if let Some(pty) = self.pty.get() {
                let _sent: Result<(), _> = pty.send(Msg::Input(text.into_bytes().into()));
            }
            return;
        }
        let _woken: Result<(), _> = self.wake.send(());
    }
}

#[derive(Clone, Copy)]
struct Size {
    columns: usize,
    lines: usize,
}

impl Dimensions for Size {
    fn total_lines(&self) -> usize {
        self.lines
    }
    fn screen_lines(&self) -> usize {
        self.lines
    }
    fn columns(&self) -> usize {
        self.columns
    }
}

fn window_size(size: Size) -> WindowSize {
    WindowSize {
        num_lines: u16::try_from(size.lines).unwrap_or(u16::MAX),
        num_cols: u16::try_from(size.columns).unwrap_or(u16::MAX),
        cell_width: 8,
        cell_height: 16,
    }
}

/// What the panel draws: one string per screen row, and the cursor.
pub(crate) struct Screen {
    pub(crate) rows: Vec<String>,
    pub(crate) cursor: Option<(usize, usize)>,
    pub(crate) app_cursor: bool,
    pub(crate) bracketed_paste: bool,
}

pub(crate) struct Session {
    term: Arc<FairMutex<Term<Proxy>>>,
    pty: EventLoopSender,
    size: Size,
    _group: Option<bb_process::OwnedGroup>,
}

impl Session {
    /// Start the user's login shell in `cwd`.
    pub(crate) fn start(
        cwd: &Path,
        columns: usize,
        lines: usize,
        wake: UnboundedSender<()>,
    ) -> std::io::Result<Session> {
        let size: Size = Size {
            columns: columns.max(2),
            lines: lines.max(1),
        };
        let proxy: Proxy = Proxy {
            wake,
            pty: Arc::new(OnceLock::new()),
        };
        let config: Config = Config {
            scrolling_history: SCROLLBACK,
            ..Config::default()
        };
        let term: Arc<FairMutex<Term<Proxy>>> =
            Arc::new(FairMutex::new(Term::new(config, &size, proxy.clone())));
        let options: tty::Options = tty::Options {
            shell: None,
            working_directory: Some(cwd.to_path_buf()),
            drain_on_exit: false,
            env: HashMap::from([("TERM".to_owned(), "xterm-256color".to_owned())]),
        };
        let pty: tty::Pty = tty::new(&options, window_size(size), 0)?;
        let group: Option<bb_process::OwnedGroup> =
            NonZeroU32::new(pty.child().id()).map(bb_process::OwnedGroup::register);
        let event_loop: EventLoop<tty::Pty, Proxy> =
            EventLoop::new(Arc::clone(&term), proxy.clone(), pty, false, false)?;
        let sender: EventLoopSender = event_loop.channel();
        let _set: Result<(), EventLoopSender> = proxy.pty.set(event_loop.channel());
        let _reader: std::thread::JoinHandle<_> = event_loop.spawn();
        Ok(Session {
            term,
            pty: sender,
            size,
            _group: group,
        })
    }

    /// Bytes typed or pasted into the shell.
    pub(crate) fn input(&self, bytes: Vec<u8>) {
        let _sent: Result<(), _> = self.pty.send(Msg::Input(bytes.into()));
    }

    /// Fit the grid and the PTY to the panel; a no-op when unchanged.
    pub(crate) fn resize(&mut self, columns: usize, lines: usize) {
        let size: Size = Size {
            columns: columns.max(2),
            lines: lines.max(1),
        };
        if (size.columns, size.lines) == (self.size.columns, self.size.lines) {
            return;
        }
        self.size = size;
        self.term.lock().resize(size);
        let _sent: Result<(), _> = self.pty.send(Msg::Resize(window_size(size)));
    }

    /// The visible rows and the cursor, as plain text.
    pub(crate) fn screen(&self) -> Screen {
        read(&self.term.lock(), self.size)
    }
}

/// The visible rows of `term` as plain text, and the cursor.
fn read(term: &Term<Proxy>, size: Size) -> Screen {
    let content: alacritty_terminal::term::RenderableContent<'_> = term.renderable_content();
    let offset: i32 = i32::try_from(content.display_offset).unwrap_or(0);
    let mut rows: Vec<Vec<char>> = vec![vec![' '; size.columns]; size.lines];
    for indexed in content.display_iter {
        if indexed.cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
            continue;
        }
        let row: Option<usize> = usize::try_from(indexed.point.line.0 + offset).ok();
        if let Some(cell) = row
            .and_then(|r: usize| rows.get_mut(r))
            .and_then(|r: &mut Vec<char>| r.get_mut(indexed.point.column.0))
        {
            *cell = indexed.cell.c;
        }
    }
    let cursor: Option<(usize, usize)> = usize::try_from(content.cursor.point.line.0 + offset)
        .ok()
        .map(|line: usize| (line, content.cursor.point.column.0));
    Screen {
        rows: rows
            .into_iter()
            .map(|row: Vec<char>| row.into_iter().collect::<String>().trim_end().to_owned())
            .collect(),
        cursor,
        app_cursor: content.mode.contains(TermMode::APP_CURSOR),
        bracketed_paste: content.mode.contains(TermMode::BRACKETED_PASTE),
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _sent: Result<(), _> = self.pty.send(Msg::Shutdown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn wait_for(session: &Session, needle: &str, within: Duration) -> bool {
        let started: Instant = Instant::now();
        while started.elapsed() < within {
            if session
                .screen()
                .rows
                .iter()
                .any(|row: &String| row.contains(needle))
            {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    /// A real shell under a real PTY: typed input runs, output is parsed.
    #[test]
    fn a_typed_command_runs_and_its_output_reaches_the_screen() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let (wake, _woken): (UnboundedSender<()>, _) = tokio::sync::mpsc::unbounded_channel();
        let mut session: Session = Session::start(dir.path(), 80, 12, wake).unwrap();
        session.input(b"echo bb_$((40+2))_ok\r".to_vec());
        assert!(
            wait_for(&session, "bb_42_ok", Duration::from_secs(10)),
            "{:?}",
            session.screen().rows
        );
        session.resize(40, 6);
        assert_eq!(session.screen().rows.len(), 6);
    }

    /// Heavy output does not block: 100,000 lines and the prompt is back fast.
    #[test]
    fn a_hundred_thousand_lines_stream_through_quickly() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let (wake, _woken): (UnboundedSender<()>, _) = tokio::sync::mpsc::unbounded_channel();
        let session: Session = Session::start(dir.path(), 100, 20, wake).unwrap();
        let started: Instant = Instant::now();
        session.input(b"seq 1 100000; echo bb_done_$((1+1))\r".to_vec());
        assert!(
            wait_for(&session, "bb_done_2", Duration::from_secs(20)),
            "{:?}",
            session.screen().rows
        );
        let took: Duration = started.elapsed();
        eprintln!("terminal_100k_lines_ms={}", took.as_millis());
        assert!(took < Duration::from_secs(10), "{took:?}");
    }
}
