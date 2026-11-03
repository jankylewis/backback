//! Group heads under a project: namespace, class, parameterised method.
//! A click folds or opens the head (BB-BUG-024); a running head's Run
//! reads Stop (BB-BUG-026).

use super::super::button;
use crate::tests_panel::Group;
use crate::{color, ExplorerView};
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, AnyElement, Context, Window};
use std::path::{Path, PathBuf};

/// A head's indent, shown name, debug selector and whether it is a class.
fn shape(group: &Group) -> (&'static str, String, String, bool) {
    match group {
        Group::Namespace(name) => ("  ", name.clone(), format!("test-ns-{name}"), false),
        Group::Class(owner) => (
            "    ",
            last_word(owner).to_owned(),
            format!("test-class-{owner}"),
            true,
        ),
        Group::Method(method) => (
            "      ",
            last_word(method).to_owned(),
            format!("test-method-{method}"),
            false,
        ),
    }
}

fn last_word(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

pub(super) fn group_row(
    view: &ExplorerView,
    path: &Path,
    group: Group,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let marker: &str = if view.tests.is_collapsed(path, &group) {
        "▸"
    } else {
        "▾"
    };
    let (indent, name, selector, is_class): (&str, String, String, bool) = shape(&group);
    let ink: bb_ui::Rgb = if is_class { theme.text } else { theme.text_dim };
    let fold_path: PathBuf = path.to_path_buf();
    let row: gpui::Stateful<gpui::Div> = div()
        .id(selector.clone())
        .debug_selector(move || selector)
        .min_h(px(24.0))
        .px_2()
        .flex()
        .justify_between()
        .cursor_pointer()
        .text_color(color(ink))
        .child(format!("{indent}{marker} {name}"));
    let row: gpui::Stateful<gpui::Div> = match &group {
        Group::Class(owner) => row.child(class_run(view, path, owner, theme, cx)),
        Group::Namespace(_) | Group::Method(_) => row,
    };
    row.on_click(cx.listener(
        move |view: &mut ExplorerView,
              _: &gpui::ClickEvent,
              _: &mut Window,
              cx: &mut Context<ExplorerView>| {
            view.tests.toggle(&fold_path, group.clone());
            cx.notify();
        },
    ))
    .into_any_element()
}

fn class_run(
    view: &ExplorerView,
    path: &Path,
    owner: &str,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let running: bool = super::any_running(view, path, &format!("{owner}."));
    let class_path: PathBuf = path.to_path_buf();
    let pair: (String, String) = match owner.rsplit_once('.') {
        Some((namespace, class)) => (namespace.to_owned(), class.to_owned()),
        None => (String::new(), owner.to_owned()),
    };
    div()
        .id("test-class-run")
        .debug_selector(|| String::from("test-class-run"))
        .child(run_or_stop(
            running,
            theme,
            cx,
            move |view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>| {
                crate::tests_panel::run_class(view, class_path.clone(), &pair, window, cx);
            },
        ))
}

/// `Run`, or `■ Stop` while the row's specs run: a hard stop that kills
/// the run's whole process group (BB-BUG-026).
pub(super) fn run_or_stop(
    running: bool,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
    run: impl Fn(&mut ExplorerView, &mut Window, &mut Context<ExplorerView>) + 'static,
) -> AnyElement {
    if running {
        return button(
            "■ Stop",
            theme,
            cx,
            |view: &mut ExplorerView,
             _: &gpui::ClickEvent,
             _: &mut Window,
             cx: &mut Context<ExplorerView>| {
                crate::tests_panel::stop(view, cx);
            },
        )
        .into_any_element();
    }
    button(
        "Run",
        theme,
        cx,
        move |view: &mut ExplorerView,
              _: &gpui::ClickEvent,
              window: &mut Window,
              cx: &mut Context<ExplorerView>| {
            run(view, window, cx);
        },
    )
    .into_any_element()
}
