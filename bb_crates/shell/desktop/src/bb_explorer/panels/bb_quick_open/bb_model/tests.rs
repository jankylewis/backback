//! The box's model in tests: the caret window test (BB-BUG-057) and Find
//! in Files' places and fields.

use std::path::{Path, PathBuf};

use super::{Chosen, QuickOpen};
use crate::intel::glue::window_tests::bb_rig::{rig, Rig};

fn setup() -> (tempfile::TempDir, PathBuf, QuickOpen) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    for file in ["Core/Core.csproj", "Specs/Specs.csproj", "Specs/Deep/X.cs"] {
        let path: PathBuf = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }
    let active: PathBuf = root.join("Specs/Deep/X.cs");
    let quick: QuickOpen = QuickOpen::find_box(&root, Some(&active), None);
    (dir, root, quick)
}

/// Find in Files' places (as Wu / VS Code / Rider): the workspace, then
/// the active file's project and folder; a folder the Explorer asked for
/// comes first.
#[test]
fn the_find_box_offers_the_workspace_the_project_and_the_folder() {
    let (_dir, root, quick): (tempfile::TempDir, PathBuf, QuickOpen) = setup();
    assert_eq!(quick.title(), "Find in Files");
    let places: Vec<String> = labels(&quick);
    assert_eq!(places.len(), 3, "{places:?}");
    assert!(places
        .get(1)
        .is_some_and(|p: &String| p.starts_with("Project · Specs")));
    assert!(places
        .get(2)
        .is_some_and(|p: &String| p.starts_with("Folder · Specs/Deep")));
    assert_eq!(quick.chosen(), None, "an empty query does nothing");
    let picked: QuickOpen = QuickOpen::find_box(&root, None, Some(&root.join("Specs")));
    assert_eq!(
        labels(&picked).first().map(String::as_str),
        Some("Folder · Specs")
    );
}

/// Tab moves to "in", whose text narrows the places, then to "files";
/// Enter asks for the text in the chosen place and files.
#[test]
fn tab_moves_to_in_and_files_and_enter_asks_for_them() {
    let (_dir, root, mut quick): (tempfile::TempDir, PathBuf, QuickOpen) = setup();
    quick.add_projects(
        &root,
        &[
            root.join("Core/Core.csproj"),
            root.join("Specs/Specs.csproj"),
        ],
    );
    assert_eq!(labels(&quick).len(), 4, "Specs once, Core added");
    quick.type_text("Greet");
    let Some(Chosen::Find(ask)): Option<Chosen> = quick.chosen() else {
        panic!("a query gives a search");
    };
    assert_eq!(
        (ask.text.as_str(), ask.dir.as_path()),
        ("Greet", root.as_path())
    );
    quick.next_field(false);
    quick.type_text("core");
    assert_eq!(labels(&quick), ["Project · Core · Core"]);
    quick.next_field(false);
    quick.type_text("*.cs, !Tests");
    let Some(Chosen::Find(ask)): Option<Chosen> = quick.chosen() else {
        panic!("still a search");
    };
    assert_eq!(ask.text, "Greet", "the text field kept its text");
    assert_eq!(ask.dir, root.join("Core"));
    assert_eq!(ask.files, "*.cs, !Tests");
    let picked: QuickOpen = QuickOpen::find_box(&root, None, Some(&root.join("Specs")));
    assert_eq!(
        labels(&picked).first().map(String::as_str),
        Some("Folder · Specs")
    );
}

fn labels(quick: &QuickOpen) -> Vec<String> {
    quick
        .matches()
        .iter()
        .map(|p: &&Path| p.display().to_string())
        .collect()
}

// BB-BUG-057: the Find in Files box takes a paste and has a caret that the
// arrow keys move, in the real window.

fn query(rig: &Rig, vcx: &mut gpui::VisualTestContext) -> (String, usize) {
    rig.view
        .read_with(vcx, |v: &crate::ExplorerView, _: &gpui::App| {
            let quick: &super::QuickOpen = v.quick_open.as_ref().unwrap();
            (quick.query.clone(), quick.caret)
        })
}

fn caret_x(vcx: &mut gpui::VisualTestContext) -> f32 {
    vcx.run_until_parked();
    f32::from(vcx.debug_bounds("quick-open-caret").unwrap().origin.x)
}

#[gpui::test]
fn the_find_box_pastes_at_a_caret_the_arrows_move(cx: &mut gpui::TestAppContext) {
    let (rig, vcx): (Rig, &mut gpui::VisualTestContext) = rig(cx, false);
    vcx.dispatch_action(crate::quick_open::FindInFiles);
    vcx.simulate_keystrokes("a b");
    vcx.write_to_clipboard(gpui::ClipboardItem::new_string("Needle\n".to_owned()));
    vcx.simulate_keystrokes("left cmd-v");
    assert_eq!(query(&rig, vcx), ("aNeedleb".to_owned(), 7));
    let end: f32 = caret_x(vcx);
    vcx.simulate_keystrokes("alt-left");
    assert_eq!(query(&rig, vcx).1, 0, "aNeedle is one word");
    let start: f32 = caret_x(vcx);
    assert!(
        start < end,
        "the caret is drawn where it is: {start} < {end}"
    );
    vcx.simulate_keystrokes("right delete x end backspace");
    assert_eq!(query(&rig, vcx), ("axeedle".to_owned(), 7));
    vcx.simulate_keystrokes("cmd-left y cmd-right z");
    assert_eq!(query(&rig, vcx), ("yaxeedlez".to_owned(), 9));
}

/// Tab and Shift+Tab reach the box in the window: keys go to "in", then
/// back to the text.
#[gpui::test]
fn tab_moves_between_the_find_fields_in_the_window(cx: &mut gpui::TestAppContext) {
    let (rig, vcx): (Rig, &mut gpui::VisualTestContext) = rig(cx, false);
    vcx.dispatch_action(crate::quick_open::FindInFiles);
    vcx.simulate_keystrokes("n tab c shift-tab x");
    let (text, place): (String, String) =
        rig.view
            .read_with(vcx, |v: &crate::ExplorerView, _: &gpui::App| {
                let quick: &QuickOpen = v.quick_open.as_ref().unwrap();
                (quick.query.clone(), quick.fields().unwrap().in_text.clone())
            });
    assert_eq!((text.as_str(), place.as_str()), ("nx", "c"));
    assert!(
        vcx.debug_bounds("quick-open-in").is_some(),
        "the in field is drawn"
    );
}

/// Rider's scope buttons keep one kind of place (Project is the project
/// dropdown); its toggles go into the search Enter asks for.
#[test]
fn scope_buttons_keep_one_kind_of_place_and_toggles_reach_the_search() {
    let (_dir, root, mut quick): (tempfile::TempDir, PathBuf, QuickOpen) = setup();
    quick.add_projects(&root, &[root.join("Core/Core.csproj")]);
    quick.set_scope(super::Scope::Project);
    assert_eq!(
        labels(&quick),
        ["Project · Specs · Specs", "Project · Core · Core"]
    );
    quick.set_scope(super::Scope::Solution);
    assert_eq!(labels(&quick).len(), 1, "only the workspace");
    quick.set_scope(super::Scope::Directory);
    assert_eq!(labels(&quick), ["Folder · Specs/Deep"]);
    quick.set_scope(super::Scope::Directory);
    assert_eq!(labels(&quick).len(), 4, "pressed again: every place");
    quick.toggle(super::Toggle::Case);
    quick.toggle(super::Toggle::Regex);
    quick.type_text("a.b");
    let Some(Chosen::Find(ask)): Option<Chosen> = quick.chosen() else {
        panic!("a search");
    };
    let expected: bb_search::Options = bb_search::Options {
        case: true,
        word: false,
        regex: true,
    };
    assert_eq!(ask.options, expected);
}

/// In the window: a click on Project and the Alt keys.
#[gpui::test]
fn the_scope_buttons_and_alt_keys_work_in_the_window(cx: &mut gpui::TestAppContext) {
    let (rig, vcx): (Rig, &mut gpui::VisualTestContext) = rig(cx, false);
    vcx.dispatch_action(crate::quick_open::FindInFiles);
    vcx.run_until_parked();
    let at: gpui::Point<gpui::Pixels> = vcx.debug_bounds("find-button-Project").unwrap().center();
    vcx.simulate_click(at, gpui::Modifiers::default());
    vcx.simulate_keystrokes("alt-w alt-x alt-x alt-c");
    let (scope, options): (super::Scope, bb_search::Options) =
        rig.view
            .read_with(vcx, |v: &crate::ExplorerView, _: &gpui::App| {
                let fields: &super::FindFields = v.quick_open.as_ref().unwrap().fields().unwrap();
                (fields.scope, fields.options)
            });
    assert_eq!(scope, super::Scope::Project);
    assert_eq!(
        (options.case, options.word, options.regex),
        (true, true, false)
    );
}
