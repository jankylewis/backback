use std::time::Duration;

use gpui::{App, AsyncWindowContext, Context, Entity, WeakEntity, Window};

use crate::harness::model::{FormField, Server, ServerStatus, ToolView};
use crate::ExplorerView;

pub(crate) fn start(view: &Entity<ExplorerView>, window: &mut Window, cx: &mut App) {
    if !eligible(view, cx) {
        return;
    }
    view.update(
        cx,
        |state: &mut ExplorerView, _: &mut Context<ExplorerView>| {
            state.harness.selected = Some(0);
            state.harness.start(0);
        },
    );
    let weak: WeakEntity<ExplorerView> = view.downgrade();
    view.update(
        cx,
        |_: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
            cx.spawn_in(
                window,
                async move |_: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
                    run_demo(weak, cx).await;
                },
            )
            .detach();
        },
    );
}

fn eligible(view: &Entity<ExplorerView>, cx: &App) -> bool {
    let fixture: Option<std::path::PathBuf> = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/_Harness/McpServer/_McpServer.csproj")
        .canonicalize()
        .ok();
    std::env::var("BB_HARNESS_DEMO").ok().as_deref() == Some("1")
        && view
            .read(cx)
            .harness
            .servers
            .first()
            .is_some_and(|server: &Server| {
                server
                    .path
                    .canonicalize()
                    .is_ok_and(|path: std::path::PathBuf| Some(path) == fixture)
            })
}

async fn run_demo(weak: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext) {
    for _ in 0..100 {
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
        let ready: Result<bool, _> = weak.update_in(
            cx,
            |state: &mut ExplorerView, _: &mut Window, cx: &mut Context<ExplorerView>| {
                let ready: bool = state
                    .harness
                    .servers
                    .first()
                    .is_some_and(|server: &Server| server.status == ServerStatus::Running);
                if ready {
                    call_greet(state);
                    cx.notify();
                }
                ready
            },
        );
        if ready.unwrap_or(false) {
            break;
        }
    }
    cx.background_executor()
        .timer(Duration::from_secs(20))
        .await;
    weak.update_in(
        cx,
        |state: &mut ExplorerView, _: &mut Window, cx: &mut Context<ExplorerView>| {
            state.harness.stop(0);
            cx.notify();
        },
    )
    .ok();
}

fn call_greet(state: &mut ExplorerView) {
    let Some(server): Option<&mut Server> = state.harness.servers.first_mut() else {
        return;
    };
    let Some(index): Option<usize> = server
        .tools
        .iter()
        .position(|tool: &ToolView| tool.name == "Greet")
    else {
        return;
    };
    let Some(field): Option<&mut FormField> =
        server.tools.get_mut(index).and_then(|tool: &mut ToolView| {
            tool.fields
                .iter_mut()
                .find(|field: &&mut FormField| field.path == ["name"])
        })
    else {
        return;
    };
    field.value = "Ada".into();
    state.harness.tool = Some(index);
    state.harness.call();
}
