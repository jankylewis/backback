use std::path::PathBuf;
use std::process::{Command as OsCommand, Stdio};
use std::time::{Duration, Instant};

use super::{Harness, ServerStatus};
use crate::harness::model::{FormField, ToolView};
use gpui::{
    point, px, size, Bounds, Modifiers, TestAppContext, VisualTestContext, WindowBounds,
    WindowOptions,
};

fn wait_for(harness: &mut Harness, wanted: impl Fn(&Harness) -> bool) {
    let deadline: Instant = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        harness.poll();
        if wanted(harness) {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("MCP fixture transition timed out");
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
            .is_some_and(|result: &String| result.contains("Ada"))
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
    while Instant::now() < deadline {
        let status: ServerStatus = view.update_in(
            visual,
            |state: &mut crate::ExplorerView,
             _: &mut gpui::Window,
             _: &mut gpui::Context<crate::ExplorerView>| {
                state.harness.poll();
                state.harness.servers[0].status.clone()
            },
        );
        if &status == wanted {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("window MCP status timed out");
}

#[gpui::test]
fn window_mouse_start_and_keystroke_edit_reach_harness(cx: &mut TestAppContext) {
    let fixture: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/_Harness");
    let window: gpui::WindowHandle<crate::ExplorerView> = cx.update(|cx: &mut gpui::App| {
        let tree: bb_explorer::Explorer = bb_explorer::Explorer::open(&fixture).unwrap();
        let harness: Harness = Harness::load(&fixture).unwrap();
        let bounds: Bounds<gpui::Pixels> = Bounds::centered(None, size(px(1000.), px(700.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window: &mut gpui::Window, cx: &mut gpui::App| {
                crate::launch::new_view(tree, harness, window.appearance(), cx)
            },
        )
        .unwrap()
    });
    let mut visual: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    let view: gpui::Entity<crate::ExplorerView> = window.root(&mut visual).unwrap();
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
