use std::path::{Path, PathBuf};

use super::bb_model::{Chosen, QuickOpen};

fn tree() -> (tempfile::TempDir, QuickOpen) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    for file in [
        "Core/Greeter.cs",
        "Core/obj/Gen.cs",
        "Specs/GreeterSpec.cs",
        "README.md",
        ".git/HEAD",
        ".github/workflows/ci.yml",
        ".idea/workspace.xml",
        "Generated/Out.cs",
    ] {
        let path: PathBuf = dir.path().join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }
    std::fs::write(dir.path().join(".gitignore"), "Generated/\n").unwrap();
    let quick: QuickOpen = QuickOpen::new(dir.path());
    (dir, quick)
}

#[test]
fn build_vcs_ide_and_ignored_files_are_never_listed_but_dot_folders_are() {
    let (_dir, quick): (tempfile::TempDir, QuickOpen) = tree();
    let all: Vec<&Path> = quick.matches();
    assert_eq!(all.len(), 5, "{all:?}");
    assert!(
        all.contains(&Path::new(".github/workflows/ci.yml")),
        "a tracked dot-folder is listed, as VS Code does (BBEOD21): {all:?}"
    );
    assert!(all.iter().all(|p: &&Path| {
        !p.starts_with(".git")
            && !p.starts_with(".idea")
            && !p.starts_with("Generated")
            && !p.to_string_lossy().contains("obj")
    }));
}

#[test]
fn a_fuzzy_query_ranks_name_matches_first_and_enter_gives_the_path() {
    let (dir, mut quick): (tempfile::TempDir, QuickOpen) = tree();
    quick.type_text("grt");
    let names: Vec<String> = quick
        .matches()
        .iter()
        .map(|p: &&Path| p.display().to_string())
        .collect();
    assert_eq!(names, ["Core/Greeter.cs", "Specs/GreeterSpec.cs"]);
    quick.step(true);
    assert_eq!(
        quick.chosen(),
        Some(Chosen::File(dir.path().join("Specs/GreeterSpec.cs")))
    );
    quick.type_text("zzz");
    assert_eq!(quick.chosen(), None);
    quick.backspace();
    quick.backspace();
    quick.backspace();
    assert_eq!(quick.matches().len(), 2);
}

#[test]
fn symbols_keep_source_order_and_enter_gives_the_line() {
    let mut quick: QuickOpen = QuickOpen::symbols(vec![
        ("class Greeter".to_owned(), 2),
        ("  method Hello".to_owned(), 4),
        ("  method Help".to_owned(), 9),
    ]);
    assert_eq!(quick.title(), "Go to Symbol");
    assert_eq!(quick.matches().len(), 3);
    quick.type_text("hel");
    quick.step(true);
    assert_eq!(quick.chosen(), Some(Chosen::Line(9)));
}

#[test]
fn the_find_box_lists_nothing_and_enter_gives_the_query() {
    let mut quick: QuickOpen = QuickOpen::find_box();
    assert_eq!(quick.title(), "Find in Files");
    assert_eq!(quick.chosen(), None, "an empty query does nothing");
    quick.type_text("Greet");
    assert_eq!(quick.matches().len(), 0);
    assert_eq!(quick.chosen(), Some(Chosen::Find("Greet".to_owned())));
}

/// BBEOD06: Cmd+Shift+F, a query, Enter: the hits fill the panel and a
/// click opens the file at the match.
#[gpui::test]
fn cmd_shift_f_searches_the_root_and_a_hit_opens_at_the_match(cx: &mut gpui::TestAppContext) {
    use crate::intel::glue::window_tests::bb_rig::{rig, Rig};
    let (rig, vcx): (Rig, &mut gpui::VisualTestContext) = rig(cx, false);
    let other: PathBuf = rig.root.join("Core/B.cs");
    std::fs::create_dir_all(other.parent().unwrap()).unwrap();
    std::fs::write(&other, "namespace Core;\n\n    var x = Needle();\n").unwrap();
    vcx.dispatch_action(super::FindInFiles);
    vcx.simulate_keystrokes("n e e d l e");
    vcx.simulate_keystrokes("enter");
    vcx.run_until_parked();
    let first: crate::intel::bb_worker::protocol::Place =
        rig.view
            .read_with(vcx, |v: &crate::ExplorerView, _: &gpui::App| {
                assert!(v.quick_open.is_none(), "Enter closes the box");
                let panel: &crate::intel::model::popups::PlacesPanel =
                    v.intel.references.as_ref().unwrap();
                assert_eq!(panel.title, "1 result for needle in 2 files");
                assert_eq!(panel.places.len(), 1);
                panel.places.first().unwrap().clone()
            });
    assert_eq!(first.location.span.start.line, 2);
    assert_eq!(first.location.span.start.character, 12);
    assert_eq!(first.preview, "var x = Needle();");
    rig.view.update_in(
        vcx,
        |v: &mut crate::ExplorerView,
         w: &mut gpui::Window,
         cx: &mut gpui::Context<crate::ExplorerView>| {
            crate::intel::glue::jump(v, &first, w, cx);
        },
    );
    vcx.run_until_parked();
    rig.view
        .read_with(vcx, |v: &crate::ExplorerView, _: &gpui::App| {
            let tab: &crate::editor::Tab = v.tabs.active_tab().unwrap();
            assert_eq!(tab.path, other.canonicalize().unwrap());
            assert_eq!(tab.document.cursor_line(), 2);
        });
}

#[gpui::test]
fn a_bad_regex_reports_and_lists_nothing(cx: &mut gpui::TestAppContext) {
    use crate::intel::glue::window_tests::bb_rig::{rig, Rig};
    let (rig, vcx): (Rig, &mut gpui::VisualTestContext) = rig(cx, false);
    vcx.dispatch_action(super::FindInFiles);
    vcx.simulate_keystrokes("/ ( /");
    vcx.simulate_keystrokes("enter");
    vcx.run_until_parked();
    rig.view
        .read_with(vcx, |v: &crate::ExplorerView, _: &gpui::App| {
            assert!(v.intel.references.is_none());
            assert!(v
                .status
                .as_deref()
                .unwrap()
                .starts_with("Find in Files: bad regex"));
        });
}

/// BB-PRF-002: Go to File walked the root on the UI thread (204 ms on
/// 50,000 files). The box opens at once; the list fills off the UI thread.
#[gpui::test]
fn go_to_file_opens_at_once_and_lists_files_off_the_ui_thread(cx: &mut gpui::TestAppContext) {
    use crate::intel::glue::window_tests::bb_rig::{rig, Rig};
    let (rig, vcx): (Rig, &mut gpui::VisualTestContext) = rig(cx, false);
    rig.view.update_in(
        vcx,
        |v: &mut crate::ExplorerView,
         w: &mut gpui::Window,
         cx: &mut gpui::Context<crate::ExplorerView>| {
            super::view::show(v, w, cx);
            let quick: &QuickOpen = v.quick_open.as_ref().unwrap();
            assert_eq!(quick.title(), "Go to File (listing files...)");
            assert_eq!(quick.matches().len(), 0, "no walk on the UI thread");
        },
    );
    vcx.run_until_parked();
    rig.view
        .read_with(vcx, |v: &crate::ExplorerView, _: &gpui::App| {
            let quick: &QuickOpen = v.quick_open.as_ref().unwrap();
            assert_eq!(quick.title(), "Go to File");
            assert_eq!(quick.matches(), [Path::new("A.cs")]);
        });
}

fn show_box(
    rig: &crate::intel::glue::window_tests::bb_rig::Rig,
    vcx: &mut gpui::VisualTestContext,
) {
    rig.view.update_in(
        vcx,
        |v: &mut crate::ExplorerView,
         w: &mut gpui::Window,
         cx: &mut gpui::Context<crate::ExplorerView>| {
            super::view::show(v, w, cx);
        },
    );
    vcx.run_until_parked();
}

fn box_open(
    rig: &crate::intel::glue::window_tests::bb_rig::Rig,
    vcx: &mut gpui::VisualTestContext,
) -> bool {
    rig.view
        .read_with(vcx, |v: &crate::ExplorerView, _: &gpui::App| {
            v.quick_open.is_some()
        })
}

/// BB-BUG-042: a press outside closes the box (focus left the box, so
/// Escape no longer reached it); a press on a row opens that file.
#[gpui::test]
fn a_press_outside_closes_the_box_and_a_row_press_opens_its_file(cx: &mut gpui::TestAppContext) {
    use crate::intel::glue::window_tests::bb_rig::{rig, Rig};
    let (rig, vcx): (Rig, &mut gpui::VisualTestContext) = rig(cx, false);
    std::fs::write(rig.root.join("B.cs"), "class B {}\n").unwrap();
    show_box(&rig, vcx);
    assert!(box_open(&rig, vcx));
    let outside: gpui::Point<gpui::Pixels> =
        vcx.debug_bounds("editor-area").unwrap().bottom_right()
            - gpui::point(gpui::px(20.0), gpui::px(20.0));
    vcx.simulate_click(outside, gpui::Modifiers::default());
    assert!(!box_open(&rig, vcx), "a press outside closed the box");
    show_box(&rig, vcx);
    let row: usize = rig
        .view
        .read_with(vcx, |v: &crate::ExplorerView, _: &gpui::App| {
            let quick: &QuickOpen = v.quick_open.as_ref().unwrap();
            quick
                .matches()
                .iter()
                .position(|p: &&Path| *p == Path::new("B.cs"))
                .unwrap()
        });
    let selector: &'static str = Box::leak(format!("quick-open-row-{row}").into_boxed_str());
    let at: gpui::Point<gpui::Pixels> = vcx.debug_bounds(selector).unwrap().center();
    vcx.simulate_click(at, gpui::Modifiers::default());
    vcx.run_until_parked();
    let active: String = rig
        .view
        .read_with(vcx, |v: &crate::ExplorerView, _: &gpui::App| {
            v.tabs
                .active_tab()
                .unwrap()
                .path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        });
    assert!(!box_open(&rig, vcx));
    assert_eq!(active, "B.cs", "the pressed row opened");
}
