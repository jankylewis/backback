//! Helpers for the BBEOD27 panel tests on real cargo: a Rust window on the
//! fixture, waits that end loudly, clicks by selector, readbacks.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bb_cargo::fixtures::{self, Shape};
use bb_lang::{Lang, LangSet};
use bb_testkit::{Fqn, Outcome, SpecResult};
use gpui::{App, Modifiers, TestAppContext, VisualTestContext, WindowHandle};

use super::super::super::open_with;
use crate::rows::cargo::CargoMembers;
use crate::ExplorerView;

pub(crate) const PASSES: &str = "alpha.tests.passes";
pub(crate) const FAILS: &str = "alpha.tests.fails";
pub(crate) const IGNORED: &str = "alpha.tests.ignored";

pub(crate) fn workspace(shape: Shape) -> (tempfile::TempDir, PathBuf) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = base.path().canonicalize().unwrap().join("ws");
    fixtures::write(&root, shape).unwrap();
    (base, root)
}

pub(crate) fn rust_window(
    cx: &mut TestAppContext,
    root: &Path,
) -> (WindowHandle<ExplorerView>, VisualTestContext) {
    let window: WindowHandle<ExplorerView> = open_with(cx, root, LangSet::of(&[Lang::Rust]));
    let vcx: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    (window, vcx)
}

/// How long a real cargo step in a window test may take before the test
/// fails, naming what it waited for and what the window showed; a step
/// blocked inside `run_until_parked` is ended by the watchdog at twice it.
pub(crate) const STEP: Duration = Duration::from_secs(90);

pub(crate) fn wait_until(
    vcx: &mut VisualTestContext,
    (window, what): (WindowHandle<ExplorerView>, &str),
    done: fn(&ExplorerView) -> bool,
) {
    let _armed: super::super::super::watchdog::Watchdog =
        super::super::super::watchdog::watchdog(what, STEP + STEP);
    let started: Instant = Instant::now();
    loop {
        vcx.run_until_parked();
        let (ok, seen): (bool, String) = window
            .read_with(vcx, |v: &ExplorerView, _: &App| {
                let projects: Vec<String> = v
                    .tests
                    .model
                    .projects()
                    .iter()
                    .map(
                        |(p, t): (&PathBuf, &crate::tests_panel::model::ProjectTests)| {
                            format!("{}:{}/{}/{:?}", p.display(), t.expanded, t.loading, t.error)
                        },
                    )
                    .collect();
                let seen: String = format!(
                    "status={:?} tests={:?} projects={projects:?}",
                    v.status, v.tests.message
                );
                (done(v), seen)
            })
            .unwrap();
        if ok {
            return;
        }
        assert!(
            started.elapsed() < STEP,
            "waited {STEP:?} for {what}; window: {seen}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

pub(crate) fn click(vcx: &mut VisualTestContext, selector: &str, mods: Modifiers) {
    let leaked: &'static str = selector.to_owned().leak();
    let bounds: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds(leaked).unwrap();
    vcx.simulate_click(bounds.center(), mods);
    vcx.run_until_parked();
}

pub(crate) fn right_click(vcx: &mut VisualTestContext, selector: &str) {
    let leaked: &'static str = selector.to_owned().leak();
    let at: gpui::Point<gpui::Pixels> = vcx.debug_bounds(leaked).unwrap().center();
    vcx.simulate_mouse_down(at, gpui::MouseButton::Right, Modifiers::default());
    vcx.simulate_mouse_up(at, gpui::MouseButton::Right, Modifiers::default());
    vcx.run_until_parked();
}

pub(crate) fn shown(vcx: &mut VisualTestContext, selector: &str) -> bool {
    let leaked: &'static str = selector.to_owned().leak();
    vcx.debug_bounds(leaked).is_some()
}

pub(crate) fn outcome(
    window: WindowHandle<ExplorerView>,
    vcx: &mut VisualTestContext,
    name: &str,
) -> Option<Outcome> {
    let fqn: Fqn = Fqn::new(name).unwrap();
    window
        .read_with(vcx, |v: &ExplorerView, _: &App| {
            v.tests.model.projects().values().find_map(
                |p: &crate::tests_panel::model::ProjectTests| {
                    p.specs
                        .get(&fqn)
                        .cloned()
                        .flatten()
                        .map(|r: SpecResult| r.outcome)
                },
            )
        })
        .unwrap()
}

pub(crate) fn members_read(v: &ExplorerView) -> bool {
    !matches!(v.rust.members, CargoMembers::Loading)
}
