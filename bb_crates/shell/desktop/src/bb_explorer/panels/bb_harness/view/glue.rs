//! Window glue: the View menu toggle, panel keys and the event poll.

use std::time::Duration;

use gpui::{prelude::*, AnyElement, Context, Window};

use super::view::{key_down, panel};
use super::HarnessVisibility;
use crate::ExplorerView;

/// Show or hide the harness panel (View menu).
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ToggleHarness;

impl ExplorerView {
    pub(crate) fn toggle_harness(
        &mut self,
        _: &ToggleHarness,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_harness = match self.show_harness {
            HarnessVisibility::Hidden => HarnessVisibility::Visible,
            HarnessVisibility::Visible => HarnessVisibility::Hidden,
        };
        cx.notify();
    }

    pub(super) fn harness_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        key_down(self, event, window, cx);
    }
}

/// The harness panel, when the View menu shows it.
pub(crate) fn visible_panel(
    view: &ExplorerView,
    cx: &mut Context<ExplorerView>,
) -> Option<AnyElement> {
    match view.show_harness {
        HarnessVisibility::Visible => Some(panel(view, cx).into_any_element()),
        HarnessVisibility::Hidden => None,
    }
}

/// Drain actor events every 100 ms and redraw when something changed.
pub(crate) fn install_poll(
    window: &mut gpui::Window,
    view: &gpui::Entity<crate::ExplorerView>,
    cx: &mut gpui::App,
) {
    let weak: gpui::WeakEntity<crate::ExplorerView> = view.downgrade();
    view.update(
        cx,
        |_: &mut crate::ExplorerView, cx: &mut gpui::Context<crate::ExplorerView>| {
            cx.spawn_in(
                window,
                async move |_: gpui::WeakEntity<crate::ExplorerView>,
                            cx: &mut gpui::AsyncWindowContext| loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(100))
                        .await;
                    let gone: bool = weak
                    .update_in(
                        cx,
                        |state: &mut crate::ExplorerView,
                         _: &mut gpui::Window,
                         cx: &mut gpui::Context<crate::ExplorerView>| {
                            if state.harness.poll() {
                                cx.notify();
                            }
                        },
                    )
                    .is_err();
                    if gone {
                        break;
                    }
                },
            )
            .detach();
        },
    );
}
