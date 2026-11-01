//! App shutdown must end the debugger and its launched debuggee.
#![cfg(test)]

#[path = "fixture.rs"]
mod fixture;
// Shared fixture helpers are exercised by the other DAP integration binaries.
#[allow(dead_code)]
#[path = "steps.rs"]
mod steps;

use std::num::NonZeroU32;
use std::time::Duration;

use bb_dap::DapSession;
use steps::{Events, Stop};

#[tokio::test]
async fn app_quit_reaps_adapter_and_debuggee() {
    let (session, mut events): (DapSession, Events) = steps::launch("spin", None).await;
    let stop: Stop = steps::first_stop(&mut events).await;
    let adapter: u32 = session.adapter_pid().unwrap();
    let debuggee: u32 = stop.process.unwrap();
    bb_process::shutdown();
    let adapter_gone: bool = steps::gone_within(adapter, Duration::from_secs(2)).await;
    let debuggee_gone: bool = steps::gone_within(debuggee, Duration::from_secs(2)).await;
    if !adapter_gone || !debuggee_gone {
        let _cleanup: std::io::Result<std::process::ExitStatus> =
            std::process::Command::new("/bin/kill")
                .args(["-KILL", &adapter.to_string(), &debuggee.to_string()])
                .status();
    }
    assert!(adapter_gone, "app quit left adapter {adapter}");
    assert!(debuggee_gone, "app quit left debuggee {debuggee}");

    let mut late: tokio::process::Child = tokio::process::Command::new("sh")
        .args(["-c", "sleep 30 & wait"])
        .process_group(0)
        .spawn()
        .unwrap();
    let leader: NonZeroU32 = NonZeroU32::new(late.id().unwrap()).unwrap();
    let group: bb_process::GroupId = bb_process::GroupId::register(leader);
    let exited: bool = tokio::time::timeout(Duration::from_millis(500), late.wait())
        .await
        .is_ok();
    let alive: bool = group.alive();
    group.end(&mut || {});
    assert!(exited && !alive, "shutdown admitted a late child group");
}
