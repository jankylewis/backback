use std::path::PathBuf;
use std::process::{Command as OsCommand, Stdio};
use std::time::{Duration, Instant};

use super::{Harness, ServerStatus};
use crate::harness::model::{FormField, RowText, Server, ToolView};
use gpui::{point, px, Modifiers, TestAppContext, VisualTestContext};

fn report(server: &Server) -> String {
    let recent: Vec<&String> = server.logs.iter().rev().take(5).collect();
    format!(
        "status {:?}, error {:?}, recent logs {recent:?}",
        server.status, server.error
    )
}

fn wait_for(harness: &mut Harness, wanted: impl Fn(&Harness) -> bool) {
    let deadline: Instant = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        harness.poll();
        if wanted(harness) {
            return;
        }
        if let ServerStatus::Crashed(error) = &harness.servers[0].status {
            panic!(
                "MCP fixture crashed: {error}; {}",
                report(&harness.servers[0])
            );
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!(
        "MCP fixture transition timed out: {}",
        report(&harness.servers[0])
    );
}

fn process_exists(pid: u32) -> bool {
    OsCommand::new("/bin/kill")
        .args(["-0", &pid.to_string()])
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success()
}

fn start_fixture(harness: &mut Harness) -> u32 {
    harness.start(0);
    assert_eq!(harness.servers[0].status, ServerStatus::Starting);
    wait_for(harness, |state: &Harness| {
        state.servers[0].status == ServerStatus::Running
    });
    wait_for(harness, |state: &Harness| {
        state.servers[0]
            .logs
            .iter()
            .any(|line: &String| line.contains("fixture boot"))
    });
    harness.servers[0].pid.unwrap()
}

fn call_greet(harness: &mut Harness) {
    let greet: usize = harness.servers[0]
        .tools
        .iter()
        .position(|tool: &ToolView| tool.name == "Greet")
        .unwrap();
    harness.tool = Some(greet);
    let field: &mut FormField = harness.servers[0].tools[greet]
        .fields
        .iter_mut()
        .find(|field: &&mut FormField| field.path == ["name"])
        .unwrap();
    field.value = "Ada".into();
    harness.call();
    wait_for(harness, |state: &Harness| {
        state.servers[0]
            .result
            .as_ref()
            .is_some_and(|result: &RowText| result.as_str().contains("Ada"))
    });
}

fn crash_mid_call(harness: &mut Harness) {
    let pid: u32 = start_fixture(harness);
    let hold: usize = harness.servers[0]
        .tools
        .iter()
        .position(|tool: &ToolView| tool.name == "Hold")
        .unwrap();
    harness.tool = Some(hold);
    harness.call();
    assert!(OsCommand::new("/bin/kill")
        .args(["-KILL", &format!("-{pid}")])
        .status()
        .unwrap()
        .success());
    wait_for(harness, |state: &Harness| {
        matches!(state.servers[0].status, ServerStatus::Crashed(_))
    });
    assert!(!process_exists(pid));
}

#[test]
fn fixture_start_greet_crash_and_stop_leave_no_process() {
    let solution: PathBuf =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/_Harness/Harness.slnx");
    let mut harness: Harness = Harness::load(&solution).unwrap();
    assert_eq!(harness.servers.len(), 1);
    assert_eq!(harness.servers[0].status, ServerStatus::Stopped);
    harness.selected = Some(0);
    let pid: u32 = start_fixture(&mut harness);
    call_greet(&mut harness);
    harness.stop(0);
    assert_eq!(harness.servers[0].status, ServerStatus::Stopping);
    wait_for(&mut harness, |state: &Harness| {
        state.servers[0].status == ServerStatus::Stopped
    });
    assert!(!process_exists(pid));
    crash_mid_call(&mut harness);
}

fn focus_name_field(view: &gpui::Entity<crate::ExplorerView>, visual: &mut VisualTestContext) {
    view.update_in(
        visual,
        |state: &mut crate::ExplorerView,
         window: &mut gpui::Window,
         cx: &mut gpui::Context<crate::ExplorerView>| {
            state.harness.selected = Some(0);
            state.harness.tool = Some(0);
            state.harness.edit = Some(0);
            state.harness.servers[0].tools = vec![ToolView {
                name: "Greet".into(),
                description: String::new(),
                fields: vec![FormField {
                    path: vec!["name".into()],
                    kind: crate::harness::model::FieldKind::String,
                    value: String::new(),
                    required: true,
                    parents: Vec::new(),
                    nullable: false,
                }],
            }];
            window.focus(&state.harness_focus, cx);
        },
    );
}

fn wait_window_status(
    view: &gpui::Entity<crate::ExplorerView>,
    visual: &mut VisualTestContext,
    wanted: &ServerStatus,
) {
    let deadline: Instant = Instant::now() + Duration::from_secs(20);
    let mut last: String = String::new();
    while Instant::now() < deadline {
        let (status, seen): (ServerStatus, String) = view.update_in(
            visual,
            |state: &mut crate::ExplorerView,
             _: &mut gpui::Window,
             _: &mut gpui::Context<crate::ExplorerView>| {
                state.harness.poll();
                let server: &Server = &state.harness.servers[0];
                (server.status.clone(), report(server))
            },
        );
        if &status == wanted {
            return;
        }
        if let ServerStatus::Crashed(error) = &status {
            panic!("window MCP server crashed: {error}; {seen}");
        }
        last = seen;
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("window MCP status {wanted:?} timed out: {last}");
}

#[gpui::test]
fn window_mouse_start_and_keystroke_edit_reach_harness(cx: &mut TestAppContext) {
    let (view, mut visual, window): render::HarnessWindow =
        render::harness_window(cx, &render::fixture_root());
    visual.simulate_click(point(px(710.), px(100.)), Modifiers::none());
    let status: ServerStatus = view
        .read_with(&visual, |state: &crate::ExplorerView, _: &gpui::App| {
            state.harness.servers[0].status.clone()
        });
    assert!(matches!(
        status,
        ServerStatus::Starting | ServerStatus::Running
    ));
    focus_name_field(&view, &mut visual);
    let stroke: gpui::Keystroke =
        gpui::Keystroke::parse_with_os("a", gpui::OperatingSystem::Mac).unwrap();
    visual.dispatch_keystroke(window.into(), stroke);
    let entered: String = view.read_with(&visual, |state: &crate::ExplorerView, _: &gpui::App| {
        state.harness.servers[0].tools[0].fields[0].value.clone()
    });
    assert_eq!(entered, "a");
    wait_window_status(&view, &mut visual, &ServerStatus::Running);
    let pid: u32 = view.read_with(&visual, |state: &crate::ExplorerView, _: &gpui::App| {
        state.harness.servers[0].pid.unwrap()
    });
    view.update_in(
        &mut visual,
        |state: &mut crate::ExplorerView,
         _: &mut gpui::Window,
         _: &mut gpui::Context<crate::ExplorerView>| state.harness.stop(0),
    );
    wait_window_status(&view, &mut visual, &ServerStatus::Stopped);
    assert!(!process_exists(pid));
}

#[path = "bb_render.rs"]
mod render;
#[path = "stop.rs"]
mod stop;
