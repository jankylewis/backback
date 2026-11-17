//! The Variables tree, as Rider's (BB-FEAT-087): a row with children shows
//! ▸; a click opens it (▾) and asks the debugger for its children. Opened
//! rows stay open across steps, by their path of names; the children are
//! asked again at each stop, since the debugger's references end there.

use std::collections::{BTreeMap, BTreeSet};

use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, AnyElement, ClickEvent, Context, Window};

use crate::run::bb_debug::session::Var;
use crate::{color, ExplorerView};

/// Indent of one tree level, in px.
const INDENT: f32 = 14.0;

/// The opened rows and the children loaded at this stop.
#[derive(Debug, Default)]
pub(crate) struct VarTree {
    /// Opened rows, by path of names (`a/b/c`).
    open: BTreeSet<String>,
    /// Each asked reference's children; `None` while they load.
    children: BTreeMap<i64, Option<Vec<Var>>>,
}

/// One shown row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    pub(crate) depth: usize,
    pub(crate) path: String,
    pub(crate) var: Var,
    pub(crate) open: bool,
}

impl VarTree {
    /// The program ran on: every reference is gone.
    pub(crate) fn forget_children(&mut self) {
        self.children.clear();
    }

    /// Open or close the row at `path`.
    pub(crate) fn toggle(&mut self, path: &str) {
        if !self.open.remove(path) {
            self.open.insert(path.to_owned());
        }
    }

    /// `reference`'s children arrived.
    pub(crate) fn loaded(&mut self, reference: i64, vars: Vec<Var>) {
        self.children.insert(reference, Some(vars));
    }

    /// The references to ask for: open rows whose children are not asked
    /// yet. Each is marked as asked.
    pub(crate) fn wanted(&mut self, roots: &[Var]) -> Vec<i64> {
        let wanted: Vec<i64> = self
            .rows(roots)
            .into_iter()
            .filter(|row: &Row| row.open && !self.children.contains_key(&row.var.reference))
            .map(|row: Row| row.var.reference)
            .collect();
        for reference in &wanted {
            self.children.insert(*reference, None);
        }
        wanted
    }

    /// The rows to show: `roots`, each open one followed by its children.
    pub(crate) fn rows(&self, roots: &[Var]) -> Vec<Row> {
        let mut rows: Vec<Row> = Vec::new();
        self.walk(roots, ("", 0), &mut rows);
        rows
    }

    fn walk(&self, vars: &[Var], (parent, depth): (&str, usize), rows: &mut Vec<Row>) {
        for var in vars {
            let path: String = if parent.is_empty() {
                var.name.clone()
            } else {
                format!("{parent}/{}", var.name)
            };
            let open: bool = var.reference > 0 && self.open.contains(&path);
            rows.push(Row {
                depth,
                path: path.clone(),
                var: var.clone(),
                open,
            });
            if let (true, Some(Some(children))) = (open, self.children.get(&var.reference)) {
                self.walk(children, (&path, depth.saturating_add(1)), rows);
            }
        }
    }
}

/// One variable row: indent, ▸ / ▾ when it has children, name, value.
pub(super) fn row(
    index: usize,
    row: Row,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let twisty: &str = match (row.var.reference > 0, row.open) {
        (false, _) => " ",
        (true, false) => "▸",
        (true, true) => "▾",
    };
    let indent: f32 = INDENT * f32::from(u16::try_from(row.depth).unwrap_or(u16::MAX));
    let path: String = row.path.clone();
    super::wide_row()
        .id(("debug-var", index))
        .debug_selector(move || format!("debug-var-{}", row.path))
        .gap_1()
        .pl(px(indent))
        .when(row.var.reference > 0, |el: gpui::Stateful<gpui::Div>| {
            el.cursor_pointer()
                .hover(|style: gpui::StyleRefinement| style.bg(color(theme.overlay)))
                .on_click(cx.listener(
                    move |view: &mut ExplorerView,
                          _: &ClickEvent,
                          _: &mut Window,
                          cx: &mut Context<ExplorerView>| {
                        view.debug.tree.toggle(&path);
                        super::super::inspect::ask_children(view);
                        cx.notify();
                    },
                ))
        })
        .child(
            div()
                .w(px(10.0))
                .text_color(color(theme.text_dim))
                .child(twisty),
        )
        .child(row.var.name)
        .child(div().text_color(color(theme.text_dim)).child("="))
        .child(div().text_color(color(theme.string)).child(row.var.value))
        .into_any_element()
}
