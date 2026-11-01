use std::path::Path;

use bb_intel::{DocPath, DocUri, IntelError, Location, Session};
use tokio::sync::mpsc::UnboundedSender;

use super::super::protocol::{Event, Place};

/// Each location with its line: from the server's mirror when the document
/// is open (it may hold unsaved text), else read from disk without blocking.
pub(super) async fn places(session: &Session, found: Vec<Location>) -> Vec<Place> {
    let mut out: Vec<Place> = Vec::with_capacity(found.len());
    for location in found {
        let text: Option<String> = match &location.uri {
            DocUri::File(path) => match session.text(path) {
                Some(open) => Some(open),
                None => tokio::fs::read_to_string(path.as_path()).await.ok(),
            },
            DocUri::Virtual(_) => None,
        };
        let preview: String = text
            .and_then(|all: String| line_of(&all, location.span.start.line))
            .unwrap_or_default();
        out.push(Place { location, preview });
    }
    out
}

fn line_of(text: &str, line: u32) -> Option<String> {
    let index: usize = usize::try_from(line).ok()?;
    text.lines()
        .nth(index)
        .map(|found: &str| found.trim().to_owned())
}

pub(super) fn doc(path: &Path) -> Result<DocPath, IntelError> {
    DocPath::new(path.to_path_buf())
}

pub(super) fn send(events: &UnboundedSender<Event>, event: Event) {
    // A closed receiver means the window is gone; nothing is left to tell.
    let _sent: Result<(), tokio::sync::mpsc::error::SendError<Event>> = events.send(event);
}
