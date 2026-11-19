//! The start page's language ticks (BBEOD26): C#, Rust and Python boxes
//! under the heading. They show the selected workspace's saved choice, else
//! what its folder holds (C# for a folder holding none); for Open... every box is ticked and means "what
//! the picked folder holds" until one is clicked. Tab moves between the
//! rows and the boxes, Space ticks the focused box; Open is disabled with
//! no box ticked.

use std::path::Path;

use bb_lang::{default_ticks, detect, Lang, LangSet, DETECT_BUDGET};
use gpui::{div, prelude::*, px, ClickEvent, Context, Window};

use super::entries::StartEntry;
use super::StartPage;
use crate::color;

/// Where the page's keys go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Area {
    Rows,
    Tick(Lang),
}

/// The boxes for `entry`: its saved ticks, else what its folder holds;
/// every language for Open....
pub(super) fn ticks_for(entry: Option<&StartEntry>) -> LangSet {
    match entry {
        Some(StartEntry::Recent(path)) => saved_or_found(path),
        Some(StartEntry::Gone(_) | StartEntry::Open) | None => LangSet::of(&Lang::ALL),
    }
}

fn saved_or_found(path: &Path) -> LangSet {
    crate::launch::recent::langs_of(path)
        .unwrap_or_else(|| default_ticks(detect(path, DETECT_BUDGET).found))
}

/// Tab order: the rows, then each box, then the rows again.
fn next_area(area: Area, back: bool) -> Area {
    let order: [Area; 4] = [
        Area::Rows,
        Area::Tick(Lang::CSharp),
        Area::Tick(Lang::Rust),
        Area::Tick(Lang::Python),
    ];
    let at: usize = order.iter().position(|a: &Area| *a == area).unwrap_or(0);
    let step: usize = if back {
        order.len().saturating_sub(1)
    } else {
        1
    };
    order
        .get(at.saturating_add(step) % order.len())
        .copied()
        .unwrap_or(Area::Rows)
}

impl StartPage {
    /// The selected entry's boxes, after a move to another entry.
    pub(super) fn refresh_ticks(&mut self) {
        self.ticks = ticks_for(self.entries.get(self.selected));
        self.touched = false;
    }

    /// Flip `lang`'s box.
    pub(crate) fn toggle_tick(&mut self, lang: Lang) {
        self.ticks = self.ticks.toggled(lang);
        self.touched = true;
    }

    /// The boxes now shown.
    pub(crate) fn ticks(&self) -> LangSet {
        self.ticks
    }

    /// What Open passes on: the boxes, or None for an untouched Open...
    /// (the picked folder's own languages).
    pub(super) fn chosen(&self, open_entry: bool) -> Option<LangSet> {
        (!open_entry || self.touched).then_some(self.ticks)
    }

    /// Tab, Shift+Tab and Space; true when the key was the ticks'.
    pub(super) fn tick_key(&mut self, key: &str, shift: bool) -> bool {
        match (key, self.area) {
            ("tab", area) => self.area = next_area(area, shift),
            ("space", Area::Tick(lang)) => self.toggle_tick(lang),
            (_, Area::Rows | Area::Tick(_)) => return false,
        }
        true
    }

    pub(super) fn ticks_row(&self, theme: bb_ui::Theme, cx: &Context<Self>) -> gpui::Div {
        let boxes: Vec<gpui::Stateful<gpui::Div>> = Lang::ALL
            .into_iter()
            .map(|lang: Lang| self.tick_box(lang, theme, cx))
            .collect();
        div()
            .debug_selector(|| String::from("start-languages"))
            .flex()
            .gap(px(16.0))
            .py(px(6.0))
            .child(div().text_color(color(theme.text_dim)).child("LANGUAGES"))
            .children(boxes)
    }

    fn tick_box(
        &self,
        lang: Lang,
        theme: bb_ui::Theme,
        cx: &Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let mark: &str = if self.ticks.has(lang) { "☑" } else { "☐" };
        let focused: bool = self.area == Area::Tick(lang);
        div()
            .id(("start-tick", lang_index(lang)))
            .debug_selector(move || format!("start-tick-{}", lang.key()))
            .px(px(4.0))
            .rounded(px(3.0))
            .border_1()
            .border_color(color(if focused { theme.accent } else { theme.bg }))
            .cursor_pointer()
            .on_click(cx.listener(
                move |page: &mut Self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>| {
                    page.toggle_tick(lang);
                    cx.notify();
                },
            ))
            .child(format!("{mark} {}", lang.name()))
    }

    /// The line under the boxes when none is ticked: why Open does nothing.
    pub(super) fn none_ticked_note(&self, theme: bb_ui::Theme) -> Option<gpui::Div> {
        self.ticks.is_empty().then(|| {
            div()
                .text_color(color(theme.text_dim))
                .child("Tick at least one language to open a workspace")
        })
    }
}

fn lang_index(lang: Lang) -> usize {
    match lang {
        Lang::CSharp => 0,
        Lang::Rust => 1,
        Lang::Python => 2,
    }
}
