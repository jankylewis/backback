//! Folding the Tests tree: BB-BUG-024 (heads fold and open) and BB-FEAT-083
//! (it starts folded below the project, as Rider).

use std::path::Path;

use bb_testkit::Fqn;
use gpui::TestAppContext;

use super::{click, shown, tree_window, Rig, NAMES};
use crate::ExplorerView;

/// BB-FEAT-083, as Rider: a project opens on its namespaces, folded; opening
/// one shows its classes folded; opening a class shows its tests.
#[gpui::test]
fn opening_a_level_shows_the_next_level_folded(cx: &mut TestAppContext) {
    let (_dir, _project, view, vcx): Rig<'_> = tree_window(cx);
    view.update(
        vcx,
        |state: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            state.tests.opened.clear();
            cx.notify();
        },
    );
    vcx.run_until_parked();
    assert!(shown(vcx, "test-ns-Sample.Auth"));
    assert!(
        !shown(vcx, "test-class-Sample.Auth.OutcomeSpec"),
        "namespace folded"
    );
    click(vcx, "test-ns-Sample.Auth");
    assert!(shown(vcx, "test-class-Sample.Auth.OutcomeSpec"));
    assert!(
        !shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Passes"),
        "class folded"
    );
    click(vcx, "test-class-Sample.Auth.OutcomeSpec");
    assert!(shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Passes"));
    assert!(shown(vcx, "test-method-Sample.Auth.OutcomeSpec.Counts"));
    assert!(
        !shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Counts(1)"),
        "a parameterised method folded"
    );
    view.update(
        vcx,
        |state: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            state.tests.opened.clear();
            state.tests.filter = String::from("Counts(2)");
            cx.notify();
        },
    );
    vcx.run_until_parked();
    assert!(
        shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Counts(2)"),
        "a typed filter shows its matches inside folded groups"
    );
}

/// Open every group, as a user would (BB-FEAT-083: they start folded).
pub(super) fn open_all(state: &mut ExplorerView, project: &Path) {
    use crate::tests_panel::Group;
    for name in NAMES {
        let fqn: Fqn = Fqn::new(name).unwrap();
        let parts: bb_testkit::FqnParts<'_> = fqn.parts();
        let owner: String = format!("{}.{}", parts.namespace, parts.class);
        for group in [
            Group::Namespace(parts.namespace.to_owned()),
            Group::Class(owner),
            Group::Method(fqn.method().to_owned()),
        ] {
            state.tests.opened.insert((project.to_path_buf(), group));
        }
    }
}

#[gpui::test]
fn namespace_class_and_method_heads_fold_and_open(cx: &mut TestAppContext) {
    let (_dir, _project, _view, vcx): Rig<'_> = tree_window(cx);
    assert!(shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Passes"));
    click(vcx, "test-class-Sample.Auth.OutcomeSpec");
    assert!(
        !shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Passes"),
        "class folded"
    );
    assert!(!shown(vcx, "test-method-Sample.Auth.OutcomeSpec.Counts"));
    assert!(shown(vcx, "test-ns-Sample.Auth"));
    click(vcx, "test-class-Sample.Auth.OutcomeSpec");
    assert!(
        shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Passes"),
        "class open"
    );
    click(vcx, "test-ns-Sample.Auth");
    assert!(
        !shown(vcx, "test-class-Sample.Auth.OutcomeSpec"),
        "namespace folded"
    );
    assert!(!shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Fails"));
    click(vcx, "test-ns-Sample.Auth");
    assert!(
        shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Fails"),
        "namespace open"
    );
    click(vcx, "test-method-Sample.Auth.OutcomeSpec.Counts");
    assert!(
        !shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Counts(1)"),
        "method folded"
    );
    assert!(shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Passes"));
}
