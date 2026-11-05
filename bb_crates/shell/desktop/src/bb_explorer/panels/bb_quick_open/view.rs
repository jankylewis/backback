//! The Go to File box: typed query, best matches, Enter opens, Esc closes.

use std::path::PathBuf;

use gpui::{
    deferred, div, prelude::*, px, AnyElement, AsyncWindowContext, Context, InteractiveElement,
    KeyDownEvent, WeakEntity, Window,
};

use super::bb_model::Chosen;
use super::QuickOpen;
use crate::{color, ExplorerView};

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct GoToFile;

/// The window-wide handlers: Go to File and Find in Files.
pub(crate) fn actions<E: InteractiveElement>(element: E, cx: &mut Context<ExplorerView>) -> E {
    element
        .on_action(cx.listener(on_go_to_file))
        .on_action(cx.listener(super::find::on_find_in_files))
}

fn on_go_to_file(
    view: &mut ExplorerView,
    _: &GoToFile,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    show(view, window, cx);
}

/// Opens the box at once; the root's files list on a background thread.
pub(crate) fn show(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    let root: PathBuf = view.root.clone();
    view.quick_open = Some(QuickOpen::listing(&root));
    window.focus(&view.quick_focus, cx);
    cx.notify();
    let walk: gpui::Task<Vec<PathBuf>> = cx
        .background_executor()
        .spawn(async move { QuickOpen::list_files(&root) });
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            let files: Vec<PathBuf> = walk.await;
            let _closed: gpui::Result<()> = view.update(
                cx,
                |v: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                    let root: PathBuf = v.root.clone();
                    if let Some(quick) = v.quick_open.as_mut() {
                        if quick.waits_for(&root) {
                            quick.fill(files);
                            cx.notify();
                        }
                    }
                },
            );
        },
    )
    .detach();
}

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct GoToSymbol;

/// Go to Symbol: the active file's classes, methods and properties (its
/// tree-sitter outline, so it works before Roslyn is ready).
pub(crate) fn on_go_to_symbol(
    view: &mut ExplorerView,
    _: &GoToSymbol,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let Some(tab): Option<&crate::editor::Tab> = view.tabs.active_tab() else {
        return;
    };
    let text: String = tab.document.text();
    let outline: Vec<(String, usize)> = bb_syntax::CSharpSyntax::outline(tab.live.tree(), &text)
        .into_iter()
        .map(|item: bb_syntax::OutlineItem| {
            let kind: &str = match item.kind {
                bb_syntax::OutlineKind::Namespace => "namespace",
                bb_syntax::OutlineKind::Type => "type",
                bb_syntax::OutlineKind::Member => "member",
            };
            (
                format!("{}{kind} {}", "  ".repeat(item.depth), item.name),
                item.line,
            )
        })
        .collect();
    view.quick_open = Some(QuickOpen::symbols(outline));
    window.focus(&view.quick_focus, cx);
    cx.notify();
}

fn key(
    view: &mut ExplorerView,
    event: &KeyDownEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    cx.stop_propagation();
    let key: &str = event.keystroke.key.as_str();
    if key == "escape" || key == "enter" {
        finish(view, key == "enter", window, cx);
        return;
    }
    let Some(quick): Option<&mut QuickOpen> = view.quick_open.as_mut() else {
        return;
    };
    match key {
        "backspace" => quick.backspace(),
        "up" => quick.step(false),
        "down" => quick.step(true),
        _ => {
            if let (Some(text), false) = (
                event.keystroke.key_char.as_deref(),
                event.keystroke.modifiers.platform,
            ) {
                quick.type_text(text);
            }
        }
    }
    cx.notify();
}

/// Close the box; with `accept`, open what is selected (Enter, a row
/// click). Escape and a press outside the box close it as is (BB-BUG-042).
fn finish(
    view: &mut ExplorerView,
    accept: bool,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let chosen: Option<Chosen> = view
        .quick_open
        .take()
        .and_then(|quick: QuickOpen| accept.then(|| quick.chosen()).flatten());
    if chosen.is_some() {
        view.tabs.leave_here();
    }
    match chosen {
        Some(Chosen::File(path)) => view.open_file(&path, window, cx),
        Some(Chosen::Line(line)) => super::find::go_to_line(view, line, window, cx),
        Some(Chosen::Find(query)) => super::find::run(view, &query, window, cx),
        Some(Chosen::Name(ask, typed)) => crate::files::apply_name(view, &ask, &typed, window, cx),
        None => {}
    }
    window.focus(&view.editor_focus, cx);
    cx.notify();
}

fn rows(
    quick: &QuickOpen,
    theme: bb_ui::bb_theme::Theme,
    cx: &mut Context<ExplorerView>,
) -> Vec<gpui::Stateful<gpui::Div>> {
    quick
        .matches()
        .into_iter()
        .enumerate()
        .map(|(index, path): (usize, &std::path::Path)| {
            div()
                .id(("quick-open-row", index))
                .debug_selector(move || format!("quick-open-row-{index}"))
                .on_click(cx.listener(
                    move |view: &mut ExplorerView,
                          _: &gpui::ClickEvent,
                          window: &mut Window,
                          cx: &mut Context<ExplorerView>| {
                        if let Some(quick) = view.quick_open.as_mut() {
                            quick.selected = index;
                        }
                        finish(view, true, window, cx);
                    },
                ))
                .px(px(8.0))
                .py(px(2.0))
                .rounded(px(3.0))
                .when(index == quick.selected, |row: gpui::Stateful<gpui::Div>| {
                    row.bg(color(theme.surface))
                })
                .child(path.display().to_string())
        })
        .collect()
}

pub(crate) fn overlay(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> Option<AnyElement> {
    let quick: &QuickOpen = view.quick_open.as_ref()?;
    let theme: bb_ui::bb_theme::Theme = view.theme;
    let rows: Vec<gpui::Stateful<gpui::Div>> = rows(quick, theme, cx);
    let query: String = format!("{}: {}", quick.title(), quick.query);
    let panel: gpui::Stateful<gpui::Div> = div()
        .id("quick-open")
        .track_focus(&view.quick_focus)
        .on_key_down(cx.listener(key))
        .on_mouse_down_out(cx.listener(
            |view: &mut ExplorerView,
             _: &gpui::MouseDownEvent,
             window: &mut Window,
             cx: &mut Context<ExplorerView>| {
                finish(view, false, window, cx);
            },
        ))
        .w(px(560.0))
        .p(px(6.0))
        .flex()
        .flex_col()
        .bg(color(theme.overlay))
        .border_1()
        .border_color(color(theme.border))
        .rounded(px(6.0))
        .child(
            div()
                .px(px(8.0))
                .py(px(4.0))
                .debug_selector(|| "quick-open-query".to_owned())
                .child(query),
        )
        .children(rows);
    Some(
        deferred(
            div()
                .absolute()
                .top(px(40.0))
                .left(px(0.0))
                .w_full()
                .flex()
                .justify_center()
                .child(panel),
        )
        .with_priority(2)
        .into_any_element(),
    )
}
