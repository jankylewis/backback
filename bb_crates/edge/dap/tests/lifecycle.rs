//! Live adapter lifecycle: adapter death, attach target exit, exit code truth.
#![cfg(test)]
#[path = "support/fixture.rs"]
mod fixture;
#[path = "support/steps.rs"]
mod steps;

use std::time::{Duration, Instant};

use bb_dap::{AdapterExit, DapError, DapSession, Debuggee, Event, ExitTruth};
use steps::{Events, Stop, Target};

#[tokio::test]
async fn adapter_death_closes_every_waiter_at_once_and_ends_the_launched_debuggee() {
    let (mut session, mut events): (DapSession, Events) = steps::launch("spin", None).await;
    let stop: Stop = steps::first_stop(&mut events).await;
    let debuggee: u32 = stop.process.unwrap();
    assert!(steps::signal("-9", session.adapter_pid().unwrap()).await);
    let started: Instant = Instant::now();
    let trace: Result<Vec<bb_dap::StackFrame>, DapError> = session.stack_trace(stop.thread).await;
    assert!(matches!(trace, Err(DapError::Closed)), "{trace:?}");
    let resumed: Result<(), DapError> = session.continue_thread(stop.thread).await;
    assert!(matches!(resumed, Err(DapError::Closed)), "{resumed:?}");
    let drained: DapError = steps::drain_to_error(&mut events).await;
    assert!(matches!(drained, DapError::Closed), "{drained:?}");
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "{:?}",
        started.elapsed()
    );
    assert!(
        steps::gone_within(debuggee, Duration::from_secs(1)).await,
        "launched debuggee {debuggee} outlived its adapter"
    );
    let exit: AdapterExit = session.wait_exit().await.unwrap();
    assert_eq!(exit.debuggee, Debuggee::Ended);
    assert_eq!(exit.code, None);
}

#[tokio::test]
async fn adapter_death_after_attach_closes_waiters_and_leaves_the_target_running() {
    let mut target: Target = Target::start().await;
    let (mut session, mut events): (DapSession, Events) = steps::start_session().await;
    session.attach(target.pid()).await.unwrap();
    session.configuration_done().await.unwrap();
    assert!(steps::signal("-9", session.adapter_pid().unwrap()).await);
    let started: Instant = Instant::now();
    let threads: Result<Vec<bb_dap::Thread>, DapError> = session.threads().await;
    assert!(matches!(threads, Err(DapError::Closed)), "{threads:?}");
    let drained: DapError = steps::drain_to_error(&mut events).await;
    assert!(matches!(drained, DapError::Closed), "{drained:?}");
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "{:?}",
        started.elapsed()
    );
    assert!(target.child.try_wait().unwrap().is_none());
    let exit: AdapterExit = session.wait_exit().await.unwrap();
    assert_eq!(exit.debuggee, Debuggee::LeftRunning(target.pid()));
    assert_eq!(target.release().await.code(), Some(5));
}

#[tokio::test]
async fn attach_target_exit_before_configuration_done_ends_the_session_within_one_second() {
    let mut target: Target = Target::start().await;
    let (session, mut events): (DapSession, Events) = steps::start_session().await;
    session.attach(target.pid()).await.unwrap();
    assert_eq!(target.release().await.code(), Some(5));
    let exited_at: Instant = Instant::now();
    let _rejected_or_done: Result<(), DapError> = session.configuration_done().await;
    loop {
        let left: Duration = Duration::from_secs(1).saturating_sub(exited_at.elapsed());
        match DapSession::next_event(&mut events, left).await {
            Ok(Event::Terminated(_)) | Err(DapError::Closed) => break,
            Ok(
                Event::Stopped(_)
                | Event::Output(_)
                | Event::Exited(_)
                | Event::Breakpoint(_)
                | Event::ConditionError { .. }
                | Event::Other(_, _),
            ) => {}
            Err(error) => panic!("no terminal event 1 s after target exit: {error:?}"),
        }
    }
}

#[tokio::test]
async fn owned_attach_exit_code_is_the_process_status() {
    let mut target: Target = Target::start().await;
    let (session, mut events): (DapSession, Events) = steps::start_session().await;
    session.attach(target.pid()).await.unwrap();
    session.configuration_done().await.unwrap();
    let status: std::process::ExitStatus = target.release().await;
    let mut adapter_code: Option<i64> = None;
    loop {
        let event: Event = DapSession::next_event(&mut events, steps::DEADLINE)
            .await
            .unwrap();
        adapter_code = adapter_code.or(event.exit_code());
        if matches!(event, Event::Terminated(_)) {
            break;
        }
    }
    let truth: ExitTruth = bb_dap::exit_truth(adapter_code, Some(status));
    eprintln!(
        "adapter_exit_code={adapter_code:?} process_exit_code={:?} truth={truth:?}",
        status.code()
    );
    assert_eq!(truth.code(), Some(5));
}

#[tokio::test]
async fn bad_condition_stop_is_a_condition_error_not_a_hit() {
    let (_session, mut events): (DapSession, Events) =
        steps::launch("spin", Some("nosuchvar > 1")).await;
    let stop: Stop = steps::first_stop(&mut events).await;
    assert!(
        matches!(&stop.event, Event::ConditionError { message, .. } if message.contains("nosuchvar")),
        "{:?}",
        stop.event
    );
}

#[tokio::test]
async fn true_condition_stop_is_a_real_hit() {
    let (_session, mut events): (DapSession, Events) =
        steps::launch("spin", Some("tick == 2")).await;
    let stop: Stop = steps::first_stop(&mut events).await;
    assert!(matches!(stop.event, Event::Stopped(_)), "{:?}", stop.event);
}

#[tokio::test]
async fn dropped_session_ends_its_launched_debuggee() {
    let (session, mut events): (DapSession, Events) = steps::launch("spin", None).await;
    let debuggee: u32 = steps::first_stop(&mut events).await.process.unwrap();
    drop(session);
    assert!(
        steps::gone_within(debuggee, Duration::from_secs(1)).await,
        "launched debuggee {debuggee} outlived its dropped session"
    );
}

#[test]
fn session_that_outlives_its_runtime_still_ends_its_launched_debuggee() {
    let runtime: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
    let (session, debuggee): (DapSession, u32) = runtime.block_on(async {
        let (session, mut events): (DapSession, Events) = steps::launch("spin", None).await;
        let pid: u32 = steps::first_stop(&mut events).await.process.unwrap();
        (session, pid)
    });
    drop(runtime);
    drop(session);
    let check: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
    assert!(
        check.block_on(steps::gone_within(debuggee, Duration::from_secs(1))),
        "launched debuggee {debuggee} outlived a session dropped after its runtime"
    );
}

#[tokio::test]
async fn real_adapter_respects_goto_capability_and_closes_waiters() {
    let (mut session, _events): (DapSession, Events) = steps::start_session().await;
    let supported: bool = session.can_set_next_statement();
    if !supported {
        let refused: Result<(), DapError> = session.goto(1, 1).await;
        assert!(
            matches!(refused, Err(DapError::Unsupported(_))),
            "{refused:?}"
        );
    }
    session.stop_adapter();
    let exit: AdapterExit = session.wait_exit().await.unwrap();
    assert_eq!(exit.debuggee, Debuggee::None);
    let threads: Result<Vec<bb_dap::Thread>, DapError> = session.threads().await;
    assert!(matches!(threads, Err(DapError::Closed)), "{threads:?}");
}

#[tokio::test]
async fn launched_exit_code_is_the_adapter_word_only_and_marked_unverified() {
    let (_session, mut events): (DapSession, Events) = steps::launch("exit3", None).await;
    let mut adapter_code: Option<i64> = None;
    loop {
        let event: Event = DapSession::next_event(&mut events, steps::DEADLINE)
            .await
            .unwrap();
        adapter_code = adapter_code.or(event.exit_code());
        if matches!(event, Event::Terminated(_)) {
            break;
        }
    }
    eprintln!("launch_exit3_adapter_exit_code={adapter_code:?}");
    let truth: ExitTruth = bb_dap::exit_truth(adapter_code, None);
    assert!(matches!(truth, ExitTruth::Unverified(_)), "{truth:?}");
}
