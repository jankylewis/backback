//! The adapter exit watcher: it owns the adapter child and is the single
//! source of session liveness.

use std::process::ExitStatus;
use std::time::Duration;

use tokio::process::Child;
use tokio::sync::{oneshot, watch};
use tokio::task::AbortHandle;

use crate::{AdapterExit, DapError, Debuggee};

use super::{group, Liveness, Target};

const REAP: Duration = Duration::from_secs(2);

/// Handle to the watcher task. Dropping it stops the adapter.
pub(in crate::bb_session) struct Watcher {
    pid: Option<u32>,
    stop: Option<oneshot::Sender<()>>,
    exit: watch::Receiver<Option<AdapterExit>>,
}

impl Watcher {
    /// Watch `child`; when it exits, stop `reader`, fail every waiter, and
    /// end a launched debuggee.
    pub(in crate::bb_session) fn spawn(child: Child, reader: AbortHandle, live: Liveness) -> Self {
        let pid: Option<u32> = child.id();
        let (stop, stop_rx): (oneshot::Sender<()>, oneshot::Receiver<()>) = oneshot::channel();
        let (exit_tx, exit): (
            watch::Sender<Option<AdapterExit>>,
            watch::Receiver<Option<AdapterExit>>,
        ) = watch::channel(None);
        let ends: Ends = Ends {
            reader,
            live,
            exit: exit_tx,
            pgid: pid,
        };
        tokio::spawn(run(child, stop_rx, ends));
        Self {
            pid,
            stop: Some(stop),
            exit,
        }
    }

    pub(in crate::bb_session) fn pid(&self) -> Option<u32> {
        self.pid
    }

    /// Ask the watcher to kill the adapter now.
    pub(in crate::bb_session) fn stop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _sent: Result<(), ()> = stop.send(());
        }
    }

    pub(in crate::bb_session) fn exit(&self) -> Option<AdapterExit> {
        self.exit.borrow().clone()
    }

    /// Wait until the adapter has exited and the cleanup is done.
    pub(in crate::bb_session) async fn wait(
        &mut self,
        limit: Duration,
    ) -> Result<AdapterExit, DapError> {
        let timeout: DapError = DapError::Timeout {
            command: "exit".to_owned(),
            after: limit,
        };
        let seen: Result<Option<AdapterExit>, watch::error::RecvError> =
            tokio::time::timeout(limit, async {
                self.exit
                    .wait_for(Option::is_some)
                    .await
                    .map(|value: watch::Ref<'_, Option<AdapterExit>>| value.clone())
            })
            .await
            .map_err(|_: _| timeout)?;
        seen.ok().flatten().ok_or(DapError::Closed)
    }
}

struct Ends {
    reader: AbortHandle,
    live: Liveness,
    exit: watch::Sender<Option<AdapterExit>>,
    pgid: Option<u32>,
}

async fn run(mut child: Child, stop: oneshot::Receiver<()>, ends: Ends) {
    let stopped: bool = tokio::select! {
        _status = child.wait() => false,
        _asked = stop => true,
    };
    if stopped {
        let _killed: std::io::Result<()> = child.start_kill();
    }
    let status: Option<ExitStatus> = tokio::time::timeout(REAP, child.wait())
        .await
        .ok()
        .and_then(Result::ok);
    ends.reader.abort();
    ends.live.close_all();
    let debuggee: Debuggee = match (ends.live.target(), ends.pgid) {
        (Target::None, _) => Debuggee::None,
        (Target::Attached(pid), _) => Debuggee::LeftRunning(pid),
        (Target::Launched, Some(pgid)) => group::end(pgid).await,
        (Target::Launched, None) => Debuggee::Unconfirmed,
    };
    let code: Option<i32> = status.and_then(|value: ExitStatus| value.code());
    let _sent: Result<(), watch::error::SendError<Option<AdapterExit>>> =
        ends.exit.send(Some(AdapterExit { code, debuggee }));
}
