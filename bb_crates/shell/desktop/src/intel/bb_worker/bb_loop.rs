//! The server thread: one tokio runtime, one [`Session`], requests handled
//! in arrival order. The UI never waits on it; answers come back as events.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bb_intel::{Config, DocPath, DocUri, IntelError, Location, Session, Span};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::progress::ProgressLog;
use super::protocol::{Answer, Ask, Event, Place, Request, Stages, Ticket};

/// How long the request queue must stay quiet before changed documents get
/// their diagnostics pulled again.
const PULL_AFTER: Duration = Duration::from_millis(250);

/// Run the server thread until the UI drops its sender.
pub(crate) fn run(
    config: &Config,
    total: usize,
    requests: UnboundedReceiver<Request>,
    events: &UnboundedSender<Event>,
) {
    let runtime: tokio::runtime::Runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            send(events, Event::Failed(format!("runtime: {error}")));
            return;
        }
    };
    runtime.block_on(serve(config, total, requests, events));
}

async fn serve(
    config: &Config,
    total: usize,
    mut requests: UnboundedReceiver<Request>,
    events: &UnboundedSender<Event>,
) {
    let log: std::fs::File = match std::fs::File::create(config.log_dir.join("backback-roslyn.log"))
    {
        Ok(file) => file,
        Err(error) => return send(events, Event::Failed(format!("run log: {error}"))),
    };
    send(events, Event::Progress { loaded: 0, total });
    let started: Result<Session, IntelError> =
        Session::start(config, ProgressLog::new(log, events.clone(), total)).await;
    let mut session: Session = match started {
        Ok(session) => session,
        Err(error) => return send(events, Event::Failed(error.to_string())),
    };
    send(
        events,
        Event::Ready {
            took: session.loaded_in(),
        },
    );
    let mut changed: BTreeSet<PathBuf> = BTreeSet::new();
    while let Some(request) = next(&mut requests, &mut session, &mut changed, events).await {
        handle(&mut session, request, &mut changed, events).await;
    }
    // The UI is gone; a failed shutdown only leaves a dead server behind.
    let _stopped: Result<(), IntelError> = session.shutdown().await;
}

/// The next request; while the queue is quiet, pull the diagnostics of
/// documents that changed.
async fn next(
    requests: &mut UnboundedReceiver<Request>,
    session: &mut Session,
    changed: &mut BTreeSet<PathBuf>,
    events: &UnboundedSender<Event>,
) -> Option<Request> {
    loop {
        if changed.is_empty() {
            return requests.recv().await;
        }
        if let Ok(request) = tokio::time::timeout(PULL_AFTER, requests.recv()).await {
            return request;
        }
        for path in std::mem::take(changed) {
            pull(session, &path, events).await;
        }
    }
}

async fn handle(
    session: &mut Session,
    request: Request,
    changed: &mut BTreeSet<PathBuf>,
    events: &UnboundedSender<Event>,
) {
    match request {
        Request::Open { path, text } => {
            let opened: Result<Vec<bb_intel::Diagnostic>, IntelError> = match doc(&path) {
                Ok(at) => session.open_text(&at, &text).await,
                Err(error) => Err(error),
            };
            match opened {
                Ok(list) => send(events, Event::Diagnostics { path, list }),
                Err(error) => send(events, Event::SyncFailed(error.to_string())),
            }
        }
        Request::Change { path, edits } => {
            let result: Result<bb_intel::DocVersion, IntelError> = match doc(&path) {
                Ok(at) => session.change(&at, &edits).await,
                Err(error) => Err(error),
            };
            match result {
                Ok(_) => {
                    changed.insert(path);
                }
                Err(error) => send(events, Event::SyncFailed(error.to_string())),
            }
        }
        Request::Close { path } => {
            changed.remove(&path);
            if let Err(error) = close(session, &path).await {
                send(events, Event::SyncFailed(error.to_string()));
            }
        }
        Request::Ask {
            ticket,
            path,
            ask,
            sent,
        } => answer(session, ticket, (path, ask), sent, events).await,
    }
}

async fn close(session: &mut Session, path: &Path) -> Result<(), IntelError> {
    session.close(&doc(path)?).await
}

async fn pull(session: &Session, path: &Path, events: &UnboundedSender<Event>) {
    let Ok(at): Result<DocPath, IntelError> = doc(path) else {
        return;
    };
    let timeout: Duration = Duration::from_secs(10);
    match session.diagnostics(&at, timeout).await {
        Ok(list) => send(
            events,
            Event::Diagnostics {
                path: path.to_path_buf(),
                list,
            },
        ),
        Err(IntelError::NotOpen(_)) => {}
        Err(error) => send(events, Event::SyncFailed(error.to_string())),
    }
}

async fn answer(
    session: &Session,
    ticket: Ticket,
    (path, ask): (PathBuf, Ask),
    sent: Instant,
    events: &UnboundedSender<Event>,
) {
    let started: Instant = Instant::now();
    let result: Result<Answer, IntelError> = match doc(&path) {
        Ok(at) => ask_server(session, &at, ask).await,
        Err(error) => Err(error),
    };
    let stages: Stages = Stages {
        sent,
        started,
        answered: Instant::now(),
    };
    let answer: Answer =
        result.unwrap_or_else(|error: IntelError| Answer::Failed(error.to_string()));
    send(
        events,
        Event::Answer {
            ticket,
            answer,
            stages,
        },
    );
}

async fn ask_server(session: &Session, at: &DocPath, ask: Ask) -> Result<Answer, IntelError> {
    let deadline: Duration = Duration::from_secs(10);
    Ok(match ask {
        Ask::Completion(point) => {
            Answer::Completion(session.completion(at, point, deadline).await?)
        }
        Ask::Hover(point) => Answer::Hover(session.hover(at, point, deadline).await?),
        Ask::Definition(point) => Answer::Definition(
            places(session, session.definition(at, point, deadline).await?).await,
        ),
        Ask::References(point) => Answer::References(
            places(session, session.references(at, point, deadline).await?).await,
        ),
        Ask::Rename(point, name) => {
            Answer::Rename(session.rename(at, point, &name, deadline).await?)
        }
        Ask::Actions(start, end) => Answer::Actions(
            session
                .code_actions(at, Span { start, end }, deadline)
                .await?,
        ),
    })
}

/// Each location with its line: from the server's mirror when the document
/// is open (it may hold unsaved text), else read from disk without blocking.
async fn places(session: &Session, found: Vec<Location>) -> Vec<Place> {
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

fn doc(path: &Path) -> Result<DocPath, IntelError> {
    DocPath::new(path.to_path_buf())
}

fn send(events: &UnboundedSender<Event>, event: Event) {
    // A closed receiver means the window is gone; nothing is left to tell.
    let _sent: Result<(), tokio::sync::mpsc::error::SendError<Event>> = events.send(event);
}
