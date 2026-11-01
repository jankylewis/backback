//! Back and Forward (Cmd+[ / Cmd+]): the places a jump left behind, as in
//! Rider and VS Code. A new jump clears Forward.

use std::path::PathBuf;

use bb_text::LspPoint;
use gpui::{Context, Window};

use crate::ExplorerView;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct GoBack;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct GoForward;

pub(crate) fn go_back(
    view: &mut ExplorerView,
    _: &GoBack,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    go(view, true, window, cx);
}

pub(crate) fn go_forward(
    view: &mut ExplorerView,
    _: &GoForward,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    go(view, false, window, cx);
}

fn go(view: &mut ExplorerView, back: bool, window: &mut Window, cx: &mut Context<ExplorerView>) {
    let Some(here): Option<Place> = view.tabs.here() else {
        return;
    };
    let to: Option<Place> = if back {
        view.tabs.history.back(here)
    } else {
        view.tabs.history.forward(here)
    };
    if let Some(place) = to {
        crate::intel::glue::goto_point(view, &place.path, place.at, window, cx);
        cx.notify();
    }
}

/// At most this many places to go back to.
const KEEP: usize = 50;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Place {
    pub(crate) path: PathBuf,
    pub(crate) at: LspPoint,
}

#[derive(Debug, Default)]
pub(crate) struct History {
    back: Vec<Place>,
    forward: Vec<Place>,
}

impl History {
    /// A jump leaves `here`: remember it for Back; Forward is gone.
    pub(crate) fn leave(&mut self, here: Place) {
        if self.back.last() != Some(&here) {
            self.back.push(here);
            if self.back.len() > KEEP {
                self.back.remove(0);
            }
        }
        self.forward.clear();
    }

    /// Where Back goes from `here`; `here` becomes Forward.
    pub(crate) fn back(&mut self, here: Place) -> Option<Place> {
        let to: Place = self.back.pop()?;
        self.forward.push(here);
        Some(to)
    }

    /// Where Forward goes from `here`; `here` becomes Back.
    pub(crate) fn forward(&mut self, here: Place) -> Option<Place> {
        let to: Place = self.forward.pop()?;
        self.back.push(here);
        Some(to)
    }
}

impl super::Tabs {
    /// The active tab's file and cursor, for Back / Forward.
    pub(crate) fn here(&self) -> Option<Place> {
        let tab: &super::Tab = self.active_tab()?;
        let at: bb_text::LspPoint = tab.document.lsp_point(tab.document.cursor()).ok()?;
        Some(Place {
            path: tab.path.clone(),
            at,
        })
    }

    /// A jump is about to leave the current place: remember it.
    pub(crate) fn leave_here(&mut self) {
        if let Some(here) = self.here() {
            self.history.leave(here);
        }
    }

    pub(crate) fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(path: &str, line: u32) -> Place {
        Place {
            path: PathBuf::from(path),
            at: LspPoint { line, character: 0 },
        }
    }

    #[test]
    fn back_and_forward_retrace_the_jumps_and_a_new_jump_clears_forward() {
        let mut history: History = History::default();
        history.leave(at("/a.cs", 1));
        history.leave(at("/b.cs", 2));
        assert_eq!(history.back(at("/c.cs", 3)), Some(at("/b.cs", 2)));
        assert_eq!(history.back(at("/b.cs", 2)), Some(at("/a.cs", 1)));
        assert_eq!(history.back(at("/a.cs", 1)), None);
        assert_eq!(history.forward(at("/a.cs", 1)), Some(at("/b.cs", 2)));
        history.leave(at("/b.cs", 2));
        assert_eq!(
            history.forward(at("/d.cs", 4)),
            None,
            "a new jump clears Forward"
        );
    }

    #[test]
    fn back_keeps_at_most_fifty_places() {
        let mut history: History = History::default();
        for line in 0..60 {
            history.leave(at("/a.cs", line));
        }
        let mut count: usize = 0;
        while history.back(at("/x.cs", 0)).is_some() {
            count += 1;
        }
        assert_eq!(count, KEEP);
    }

    fn active(
        rig: &crate::intel::glue::window_tests::bb_rig::Rig,
        vcx: &mut gpui::VisualTestContext,
    ) -> PathBuf {
        rig.view
            .read_with(vcx, |v: &crate::ExplorerView, _: &gpui::App| {
                v.tabs.active_tab().unwrap().path.clone()
            })
    }

    /// F12 into another file, then Back returns to the first file and line,
    /// and Forward goes to the target again (the real actions on the editor).
    #[gpui::test]
    fn back_and_forward_retrace_an_f12_jump_in_the_window(cx: &mut gpui::TestAppContext) {
        use crate::intel::bb_worker::protocol::{Answer, Ticket};
        use crate::intel::glue::window_tests::bb_rig::{place, rig, Rig};
        let (mut rig, vcx): (Rig, &mut gpui::VisualTestContext) = rig(cx, false);
        let other: PathBuf = rig.root.join("Other.cs");
        std::fs::write(&other, "line0\nclass Other {}\n").unwrap();
        vcx.simulate_keystrokes("f12");
        let (ticket, _): (Ticket, _) = rig.last_ask();
        rig.reply(vcx, ticket, Answer::Definition(vec![place(&other, 1)]));
        assert_eq!(active(&rig, vcx), other);
        vcx.dispatch_action(GoBack);
        assert_eq!(
            active(&rig, vcx),
            rig.file,
            "Back returns to the first file"
        );
        vcx.dispatch_action(GoForward);
        assert_eq!(active(&rig, vcx), other, "Forward goes to the target again");
    }
}
