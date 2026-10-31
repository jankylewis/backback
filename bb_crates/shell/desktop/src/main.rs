//! macOS desktop shell for the backback Explorer.

mod icons;
mod navigation;
mod tooltip;

use std::{path::PathBuf, time::Instant};

use bb_explorer::{Entry, Explorer};
use bb_ui::{theme::DARK, Rgb};
use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, Context, FocusHandle, InteractiveElement,
    IntoElement, KeyBinding, Render, StatefulInteractiveElement, Window, WindowBounds,
    WindowOptions,
};
use icons::{chevron, icon};

/// Select the preceding Explorer row.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
struct MoveUp;
/// Select the next Explorer row.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
struct MoveDown;
/// Collapse the selected folder or select its parent.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
struct MoveLeft;
/// Expand the selected folder.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
struct MoveRight;
/// Open the selected entry.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
struct Open;

struct ExplorerView {
    tree: Explorer,
    selected: usize,
    focus: FocusHandle,
}

impl ExplorerView {
    fn move_up(&mut self, _: &MoveUp, _: &mut Window, cx: &mut Context<Self>) {
        self.selected = self.selected.saturating_sub(1);
        cx.notify();
    }

    fn move_down(&mut self, _: &MoveDown, _: &mut Window, cx: &mut Context<Self>) {
        self.selected = self
            .selected
            .saturating_add(1)
            .min(self.tree.visible().len().saturating_sub(1));
        cx.notify();
    }

    fn move_left(&mut self, _: &MoveLeft, _: &mut Window, cx: &mut Context<Self>) {
        match navigation::left(&mut self.tree, self.selected) {
            Ok(selected) => self.selected = selected,
            Err(error) => eprintln!("{error}"),
        }
        cx.notify();
    }

    fn move_right(&mut self, _: &MoveRight, _: &mut Window, cx: &mut Context<Self>) {
        match navigation::right(&mut self.tree, self.selected) {
            Ok(selected) => self.selected = selected,
            Err(error) => eprintln!("{error}"),
        }
        cx.notify();
    }

    fn open(&mut self, _: &Open, _: &mut Window, cx: &mut Context<Self>) {
        self.open_selected(cx);
    }

    fn open_selected(&mut self, cx: &mut Context<Self>) {
        let path: Option<PathBuf> = self
            .tree
            .visible()
            .get(self.selected)
            .map(|(_, entry): &(usize, &Entry)| entry.path().to_path_buf());
        if let Some(path) = path {
            if let Err(error) = self.tree.toggle(&path) {
                eprintln!("{error}");
            }
        }
        cx.notify();
    }
}

impl Render for ExplorerView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows: Vec<(usize, &Entry)> = self.tree.visible();
        div()
            .size_full()
            .flex()
            .bg(color(DARK.bg))
            .child(
                div()
                    .id("explorer-panel")
                    .w(px(300.0))
                    .h_full()
                    .flex()
                    .flex_col()
                    .bg(color(DARK.surface))
                    .overflow_y_scroll()
                    .key_context("Explorer")
                    .track_focus(&self.focus)
                    .on_action(cx.listener(Self::move_up))
                    .on_action(cx.listener(Self::move_down))
                    .on_action(cx.listener(Self::move_left))
                    .on_action(cx.listener(Self::move_right))
                    .on_action(cx.listener(Self::open))
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .text_color(color(DARK.text))
                            .child("EXPLORER"),
                    )
                    .children(rows.into_iter().enumerate().map(
                        |(index, (depth, entry)): (usize, (usize, &Entry))| {
                            row(index, depth, entry, index == self.selected, cx)
                        },
                    )),
            )
            .child(div().flex_1().h_full().child(""))
    }
}

fn row(
    index: usize,
    depth: usize,
    entry: &Entry,
    selected: bool,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let path: PathBuf = entry.path().to_path_buf();
    let tooltip_path: String = path.display().to_string();
    let expandable: bool = entry.expandable();
    let icon: &str = icon(entry.kind());
    let name: String = entry.name().to_owned();
    let text_color: gpui::Rgba = color(if entry.dimmed() {
        DARK.text_dim
    } else {
        DARK.text
    });
    let depth_px: f32 = u16::try_from(depth).map_or(f32::from(u16::MAX), f32::from);
    div()
        .id(index)
        .pl(px(8.0 + depth_px * 16.0))
        .py_1()
        .flex()
        .gap_2()
        .text_color(text_color)
        .border_1()
        .border_color(color(if selected { DARK.accent } else { DARK.surface }))
        .hover(|style: gpui::StyleRefinement| style.bg(color(DARK.overlay)))
        .tooltip(tooltip::path(tooltip_path))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  _: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                view.selected = index;
                if expandable {
                    if let Err(error) = view.tree.toggle(&path) {
                        eprintln!("{error}");
                    }
                }
                cx.notify();
            },
        ))
        .child(chevron(entry))
        .child(icon)
        .child(name)
}

fn color(value: Rgb) -> gpui::Rgba {
    let packed: u32 = (u32::from(value.r) << 16) | (u32::from(value.g) << 8) | u32::from(value.b);
    rgb(packed)
}

fn main() {
    let start: Instant = Instant::now();
    let path: PathBuf = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    let tree: Explorer = match Explorer::open(&path) {
        Ok(tree) => tree,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    gpui_platform::application().run(move |cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("up", MoveUp, Some("Explorer")),
            KeyBinding::new("down", MoveDown, Some("Explorer")),
            KeyBinding::new("left", MoveLeft, Some("Explorer")),
            KeyBinding::new("right", MoveRight, Some("Explorer")),
            KeyBinding::new("enter", Open, Some("Explorer")),
        ]);
        let bounds: Bounds<gpui::Pixels> = Bounds::centered(None, size(px(1000.0), px(700.0)), cx);
        let result: gpui::Result<gpui::WindowHandle<ExplorerView>> = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window: &mut Window, cx: &mut App| {
                let view: gpui::Entity<ExplorerView> =
                    cx.new(|cx: &mut gpui::Context<ExplorerView>| ExplorerView {
                        tree,
                        selected: 0,
                        focus: cx.focus_handle(),
                    });
                let focus: FocusHandle = view.read(cx).focus.clone();
                window.focus(&focus, cx);
                window.on_next_frame(move |_: &mut Window, _: &mut App| {
                    eprintln!("first_frame_ms={}", start.elapsed().as_millis());
                });
                view
            },
        );
        if let Err(error) = result {
            eprintln!("{error}");
            cx.quit();
        }
        cx.activate(true);
    });
}
