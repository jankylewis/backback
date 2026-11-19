//! BBEOD33 item 3 (BBEOD32 row 1a): the start page's RECENT heading sits on
//! its list, the list scrolls to its last row by keys and wheel, and a
//! folder gone from disk shows greyed with a remove button.

use std::path::{Path, PathBuf};

use gpui::{App, Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext, WindowHandle};

use super::super::bb_start::{open, StartPage};
use crate::launch::recent::{list, remember, test_store};

fn folders(base: &Path, count: usize) -> Vec<PathBuf> {
    (0..count)
        .map(|n: usize| {
            let path: PathBuf = base.join(format!("w{n}"));
            std::fs::create_dir_all(&path).unwrap();
            path.canonicalize().unwrap()
        })
        .collect()
}

fn page_for(
    cx: &mut TestAppContext,
    roots: Vec<PathBuf>,
) -> (WindowHandle<StartPage>, VisualTestContext) {
    let page: WindowHandle<StartPage> = cx.update(|cx: &mut App| open(roots, false, cx)).unwrap();
    let vcx: VisualTestContext = VisualTestContext::from_window(page.into(), cx);
    vcx.run_until_parked();
    (page, vcx)
}

fn bounds(vcx: &mut VisualTestContext, selector: &str) -> Bounds<Pixels> {
    vcx.debug_bounds(selector.to_owned().leak()).unwrap()
}

fn inside(row: Bounds<Pixels>, list: Bounds<Pixels>) -> bool {
    row.top() >= list.top() - gpui::px(0.5) && row.bottom() <= list.bottom() + gpui::px(0.5)
}

#[gpui::test]
fn the_recent_heading_sits_directly_above_its_list(cx: &mut TestAppContext) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let (_page, mut vcx): (WindowHandle<StartPage>, VisualTestContext) =
        page_for(cx, folders(base.path(), 2));
    let languages: Bounds<Pixels> = bounds(&mut vcx, "start-languages");
    let heading: Bounds<Pixels> = bounds(&mut vcx, "start-recent-heading");
    let rows: Bounds<Pixels> = bounds(&mut vcx, "start-rows");
    assert!(
        languages.bottom() <= heading.top(),
        "LANGUAGES {languages:?} above RECENT {heading:?}"
    );
    assert!(
        heading.bottom() <= rows.top(),
        "RECENT {heading:?} above its rows {rows:?}"
    );
    assert!(
        rows.top() - heading.bottom() <= gpui::px(8.0),
        "nothing between: {heading:?} {rows:?}"
    );
}

#[gpui::test]
fn the_last_row_scrolls_into_view_by_keys_and_by_wheel(cx: &mut TestAppContext) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let (_page, mut vcx): (WindowHandle<StartPage>, VisualTestContext) =
        page_for(cx, folders(base.path(), 10));
    vcx.simulate_resize(gpui::size(gpui::px(640.0), gpui::px(360.0)));
    vcx.run_until_parked();
    let window: Bounds<Pixels> = bounds(&mut vcx, "start-page-root");
    let list: Bounds<Pixels> = bounds(&mut vcx, "start-rows");
    assert!(
        list.bottom() <= window.bottom(),
        "the list ends inside the window: {list:?} {window:?}"
    );
    assert!(
        !inside(bounds(&mut vcx, "start-entry-10"), list),
        "Open... starts out of sight"
    );
    for _ in 0..10 {
        vcx.simulate_keystrokes("down");
    }
    vcx.run_until_parked();
    let open_row: Bounds<Pixels> = bounds(&mut vcx, "start-entry-10");
    assert!(
        inside(open_row, list),
        "Down to Open... shows it whole: {open_row:?} in {list:?}"
    );
    for _ in 0..10 {
        vcx.simulate_keystrokes("up");
    }
    vcx.run_until_parked();
    assert!(
        inside(bounds(&mut vcx, "start-entry-0"), list),
        "Up to the first row shows it"
    );
    vcx.simulate_event(gpui::ScrollEvent {
        position: list.center(),
        delta: gpui::ScrollDelta::Pixels(gpui::point(gpui::px(0.0), gpui::px(-2000.0))),
        ..Default::default()
    });
    vcx.run_until_parked();
    assert!(
        inside(bounds(&mut vcx, "start-entry-10"), list),
        "the wheel reaches Open..."
    );
}

#[gpui::test]
fn a_gone_folder_shows_greyed_and_its_remove_button_drops_it_from_the_list_only(
    cx: &mut TestAppContext,
) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let store: tempfile::TempDir = tempfile::tempdir().unwrap();
    test_store(Some(store.path().to_path_buf()));
    let kept: Vec<PathBuf> = folders(base.path(), 2);
    let gone: PathBuf = base.path().canonicalize().unwrap().join("gone");
    std::fs::create_dir(&gone).unwrap();
    for path in [&kept[1], &gone, &kept[0]] {
        remember(path);
    }
    std::fs::remove_dir(&gone).unwrap();
    let (page, mut vcx): (WindowHandle<StartPage>, VisualTestContext) = page_for(cx, list());
    let labels: Vec<String> = page
        .read_with(&vcx, |p: &StartPage, _: &App| p.labels())
        .unwrap();
    let gone_label: String = format!("gone {} (no longer exists)", gone.display());
    assert_eq!(labels.get(1), Some(&gone_label), "{labels:?}");
    assert!(
        vcx.debug_bounds("start-gone-1").is_some(),
        "the gone row is greyed"
    );
    assert!(
        vcx.debug_bounds("start-gone-0").is_none(),
        "a folder on disk is not"
    );
    vcx.simulate_keystrokes("down enter");
    vcx.run_until_parked();
    assert_eq!(
        cx.windows().len(),
        1,
        "Enter on a gone folder opens nothing"
    );
    let remove: Bounds<Pixels> = bounds(&mut vcx, "start-remove-1");
    vcx.simulate_click(remove.center(), Modifiers::default());
    vcx.run_until_parked();
    let after: Vec<String> = page
        .read_with(&vcx, |p: &StartPage, _: &App| p.labels())
        .unwrap();
    assert_eq!(after.len(), 3, "{after:?}");
    assert!(!list().contains(&gone), "removed from the saved list");
    assert_eq!(list(), vec![kept[0].clone(), kept[1].clone()]);
    assert!(
        kept.iter().all(|path: &PathBuf| path.is_dir()),
        "nothing on disk is touched"
    );
    test_store(None);
}
