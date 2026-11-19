//! The Cargo explorer (BBEOD27): above the file tree of a window with Rust
//! active, the workspace's members from `cargo metadata --no-deps` (read
//! in the background, with a deadline); a click opens a member's
//! `Cargo.toml`. A failed read says why there, never in a popup: one line
//! with the place cargo named, the full text on hover (BBEOD33).

use std::path::PathBuf;

use bb_cargo::{CargoError, CargoWorkspace, Member, METADATA_DEADLINE};
use gpui::prelude::*;
use gpui::{div, px, AnyElement, AsyncWindowContext, Context, WeakEntity, Window};

use crate::theme::color;
use crate::ExplorerView;

/// What the section shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CargoMembers {
    /// Rust is not active, or the folder has no `Cargo.toml`.
    Off,
    /// `cargo metadata` runs.
    Loading,
    /// The members, by name.
    Ready(Vec<Member>),
    /// The read failed: one line, and cargo's full text.
    Failed { summary: String, full: String },
}

/// Read the members while Rust is active and the root holds a
/// `Cargo.toml`.
pub(crate) fn load(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    let root: Option<PathBuf> = crate::intel::bb_worker::find_cargo(&view.root)
        .filter(|_: &PathBuf| view.langs.start(bb_lang::Tool::Cargo));
    let Some(root): Option<PathBuf> = root else {
        return;
    };
    view.rust.members = CargoMembers::Loading;
    let job: gpui::Task<Result<CargoWorkspace, CargoError>> = cx
        .background_executor()
        .spawn(async move { bb_cargo::members(&root, METADATA_DEADLINE) });
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            let found: Result<CargoWorkspace, CargoError> = job.await;
            let _closed: gpui::Result<()> = view.update(
                cx,
                |v: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                    v.rust.members = match found {
                        Ok(workspace) => CargoMembers::Ready(workspace.members),
                        Err(error) => CargoMembers::Failed {
                            summary: error.summary(),
                            full: error.to_string(),
                        },
                    };
                    cx.notify();
                },
            );
        },
    )
    .detach();
}

/// The section, or nothing while off.
pub(crate) fn section(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> Option<AnyElement> {
    let theme: bb_ui::Theme = view.theme;
    let rows: Vec<AnyElement> = match &view.rust.members {
        CargoMembers::Off => return None,
        CargoMembers::Loading => vec![note("Reading cargo metadata...", theme)],
        CargoMembers::Failed { summary, full } => vec![failure(summary, full, theme)],
        CargoMembers::Ready(members) => members
            .iter()
            .map(|member: &Member| row(member, theme, cx))
            .collect(),
    };
    Some(
        div()
            .id("cargo-members")
            .debug_selector(|| "cargo-members".to_owned())
            .px_3()
            .pb_2()
            .flex()
            .flex_col()
            .child(note("CARGO WORKSPACE", theme))
            .children(rows)
            .into_any_element(),
    )
}

fn note(text: &str, theme: bb_ui::Theme) -> AnyElement {
    div()
        .text_color(color(theme.text_dim))
        .child(text.to_owned())
        .into_any_element()
}

/// The failure's one line, cut with an ellipsis; a hover shows `full`.
fn failure(summary: &str, full: &str, theme: bb_ui::Theme) -> AnyElement {
    div()
        .id("cargo-error")
        .debug_selector(|| "cargo-error".to_owned())
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .text_color(color(theme.error))
        .child(summary.to_owned())
        .tooltip(crate::tooltip::path(full.to_owned(), theme))
        .into_any_element()
}

fn row(member: &Member, theme: bb_ui::Theme, cx: &mut Context<ExplorerView>) -> AnyElement {
    let selector: String = format!("cargo-member-{}", member.name);
    let manifest: PathBuf = member.manifest.clone();
    div()
        .id(selector.clone())
        .debug_selector(move || selector)
        .min_h(px(22.0))
        .pl_2()
        .cursor_pointer()
        .text_color(color(theme.text))
        .hover(move |style: gpui::StyleRefinement| style.bg(color(theme.overlay)))
        .child(format!("📦 {}", member.name))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                view.open_file(&manifest, window, cx);
            },
        ))
        .into_any_element()
}
