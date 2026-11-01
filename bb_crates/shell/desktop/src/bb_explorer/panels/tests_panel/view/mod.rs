use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, AnyElement, Context, IntoElement, Window};

use crate::{color, ExplorerView};

#[path = "bb_controls.rs"]
mod controls;
mod rows;
use rows::{project_row, spec_rows};
mod detail;
use controls::filter_key;
use detail::{detail, label};

pub(crate) fn panel(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    let theme: Theme = view.theme;
    let rows: Vec<AnyElement> = project_rows(view, theme, cx);
    div()
        .id("tests-panel")
        .w(px(view.panel_width))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .bg(color(theme.surface))
        .text_color(color(theme.text))
        .track_focus(&view.focus)
        .on_key_down(cx.listener(filter_key))
        .child(header(view, theme, cx))
        .child(toolbar(view, theme, cx))
        .child(filter(view, theme, cx))
        .child(settings_picker(view, cx))
        .child(
            div()
                .id("test-rows")
                .flex_1()
                .overflow_y_scroll()
                .children(rows),
        )
        .child(detail(view, theme, cx))
}

fn project_rows(
    view: &ExplorerView,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> Vec<AnyElement> {
    let mut rows: Vec<AnyElement> = Vec::new();
    if view.tests.scan == super::Scan::Running {
        rows.push(label("Discovering projects…", theme));
    }
    for (path, project) in view.tests.model.projects() {
        rows.push(project_row(
            path.clone(),
            project,
            view.tests.model.counts(path),
            theme,
            cx,
        ));
        if project.expanded {
            if project.loading {
                rows.push(label("  Discovering tests…", theme));
            } else if let Some(error) = &project.error {
                rows.push(label(format!("  {error}"), theme));
            } else {
                rows.extend(spec_rows(view, path, project, theme, cx));
            }
        }
    }
    rows
}

fn filter(view: &ExplorerView, theme: Theme, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .id("test-filter")
        .px_2()
        .min_h(px(28.0))
        .bg(color(theme.overlay))
        .child(format!("Filter: {}", view.tests.filter))
        .on_click(cx.listener(
            |view: &mut ExplorerView,
             _: &gpui::ClickEvent,
             window: &mut Window,
             cx: &mut Context<ExplorerView>| {
                view.tests.input = super::Input::Filter;
                window.focus(&view.focus, cx);
                cx.notify();
            },
        ))
}

fn settings_picker(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .id("runsettings-picker")
        .px_2()
        .child(format!("Runsettings: {}", view.tests.settings_label()))
        .on_click(cx.listener(
            |view: &mut ExplorerView,
             _: &gpui::ClickEvent,
             _: &mut Window,
             cx: &mut Context<ExplorerView>| {
                view.tests.settings_index =
                    (view.tests.settings_index + 1) % (view.tests.settings_choices.len() + 1);
                cx.notify();
            },
        ))
}

fn header(view: &ExplorerView, theme: Theme, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .px_2()
        .py_2()
        .flex()
        .justify_between()
        .child("TESTS")
        .child(button(
            "Files",
            theme,
            cx,
            |view: &mut ExplorerView,
             _: &gpui::ClickEvent,
             _: &mut Window,
             cx: &mut Context<ExplorerView>| {
                view.tests.side = super::Side::Files;
                cx.notify();
            },
        ))
        .child(if view.tests.busy.is_some() {
            "Running…"
        } else {
            ""
        })
}

fn toolbar(view: &ExplorerView, theme: Theme, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .px_2()
        .flex()
        .gap_2()
        .child(button(
            "Run failed",
            theme,
            cx,
            |view: &mut ExplorerView,
             _: &gpui::ClickEvent,
             window: &mut Window,
             cx: &mut Context<ExplorerView>| {
                super::run_failed(view, window, cx);
            },
        ))
        .child(button(
            "Stop",
            theme,
            cx,
            |view: &mut ExplorerView,
             _: &gpui::ClickEvent,
             _: &mut Window,
             cx: &mut Context<ExplorerView>| {
                super::stop(view, cx);
            },
        ))
        .child(toolbar_extra(view, theme, cx))
}

fn toolbar_extra(
    view: &ExplorerView,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    div()
        .flex()
        .gap_2()
        .child(button(
            "Category",
            theme,
            cx,
            |view: &mut ExplorerView,
             _: &gpui::ClickEvent,
             window: &mut Window,
             cx: &mut Context<ExplorerView>| {
                view.tests.input = super::Input::Category;
                window.focus(&view.focus, cx);
                cx.notify();
            },
        ))
        .child(button(
            "Output",
            theme,
            cx,
            |view: &mut ExplorerView,
             _: &gpui::ClickEvent,
             _: &mut Window,
             cx: &mut Context<ExplorerView>| {
                view.tests.output_open = !view.tests.output_open;
                cx.notify();
            },
        ))
        .child(if view.tests.input == super::Input::Category {
            format!("{}▌", view.tests.category)
        } else {
            view.tests.category.clone()
        })
}

pub(super) fn button(
    title: &'static str,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
    action: impl Fn(&mut ExplorerView, &gpui::ClickEvent, &mut Window, &mut Context<ExplorerView>)
        + 'static,
) -> impl IntoElement {
    div()
        .id(title)
        .cursor_pointer()
        .text_color(color(theme.accent))
        .child(title)
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  event: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                cx.stop_propagation();
                action(view, event, window, cx);
            },
        ))
}
