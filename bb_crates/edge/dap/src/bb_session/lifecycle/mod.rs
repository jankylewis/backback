//! Session lifecycle: setup order, liveness from the adapter process exit,
//! attach target exit, and debuggee cleanup.
//!
//! Liveness never comes from stdout EOF alone: a launched debuggee inherits
//! the adapter's stdout, so the pipe can stay open after the adapter dies.

mod attach;
mod bb_exit;
mod group;
pub(super) mod state;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::DapError;

pub(super) use attach::watch_target;
pub(super) use bb_exit::Watcher;
pub(super) use group::kill_group;

use super::{Pending, Reply};

/// What the session started or joined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Target {
    None,
    Launched,
    Attached(u32),
}

/// State shared by the reader, the exit watcher, the attach poller and requests.
#[derive(Clone)]
pub(super) struct Liveness {
    pub(super) pending: Pending,
    closed: Arc<AtomicBool>,
    terminated: Arc<AtomicBool>,
    target: Arc<Mutex<Target>>,
}

impl Liveness {
    pub(super) fn new() -> Self {
        Self {
            pending: Pending::default(),
            closed: Arc::new(AtomicBool::new(false)),
            terminated: Arc::new(AtomicBool::new(false)),
            target: Arc::new(Mutex::new(Target::None)),
        }
    }

    pub(super) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }

    pub(super) fn saw_terminated(&self) {
        self.terminated.store(true, Ordering::Release);
    }

    fn is_terminated(&self) -> bool {
        self.terminated.load(Ordering::Acquire)
    }

    pub(super) fn target(&self) -> Target {
        self.target
            .lock()
            .map_or(Target::None, |guard: std::sync::MutexGuard<'_, Target>| {
                *guard
            })
    }

    pub(super) fn set_target(&self, target: Target) -> Result<(), DapError> {
        *self.target.lock().map_err(|_: _| DapError::Closed)? = target;
        Ok(())
    }

    /// Mark the session closed and fail every waiter with [`DapError::Closed`].
    pub(super) fn close_all(&self) {
        self.closed.store(true, Ordering::Release);
        if let Ok(mut map) = self.pending.lock() {
            for (_, tx) in map.drain() {
                let _sent: Result<(), Reply> = tx.send(Err(DapError::Closed));
            }
        }
    }
}
