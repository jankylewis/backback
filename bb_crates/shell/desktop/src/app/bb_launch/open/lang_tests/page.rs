//! The start page's boxes: mouse and keys, none ticked, saved, File > Languages....

use std::path::PathBuf;

use bb_lang::fixtures::FIXTURES;
use bb_lang::{Lang, LangSet};
use gpui::{AnyWindowHandle, App, Modifiers, TestAppContext, VisualTestContext, WindowHandle};

use super::super::bb_start::{open, StartPage};
use super::{close_all, open_with, opened};
use crate::launch::recent::test_store;
use crate::ExplorerView;

fn page_for(
    cx: &mut TestAppContext,
    roots: Vec<PathBuf>,
) -> (WindowHandle<StartPage>, VisualTestContext) {
    let page: WindowHandle<StartPage> = cx.update(|cx: &mut App| open(roots, false, cx)).unwrap();
    let vcx: VisualTestContext = VisualTestContext::from_window(page.into(), cx);
    (page, vcx)
}

fn page_ticks(page: WindowHandle<StartPage>, vcx: &VisualTestContext) -> LangSet {
    page.read_with(vcx, |p: &StartPage, _: &App| p.ticks())
        .unwrap()
}

fn click_tick(vcx: &mut VisualTestContext, lang: Lang) {
    let bounds: gpui::Bounds<gpui::Pixels> = vcx
        .debug_bounds(format!("start-tick-{}", lang.key()).leak())
        .unwrap();
    vcx.simulate_click(bounds.center(), Modifiers::default());
    vcx.run_until_parked();
}

#[gpui::test]
fn the_start_page_ticks_by_mouse_and_keys_and_opens_each_fixture_with_them(
    cx: &mut TestAppContext,
) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    for fixture in &FIXTURES {
        let root: PathBuf = fixture.write(base.path()).unwrap();
        let (page, mut vcx): (WindowHandle<StartPage>, VisualTestContext) =
            page_for(cx, vec![root.clone()]);
        assert_eq!(
            page_ticks(page, &vcx),
            LangSet::of(fixture.langs),
            "{} defaults",
            fixture.name
        );
        click_tick(&mut vcx, Lang::Rust);
        vcx.simulate_keystrokes("tab tab tab space tab");
        let want: LangSet = LangSet::of(fixture.langs)
            .toggled(Lang::Rust)
            .toggled(Lang::Python);
        assert_eq!(page_ticks(page, &vcx), want, "{}", fixture.name);
        vcx.simulate_keystrokes("enter");
        vcx.run_until_parked();
        let windows: Vec<WindowHandle<ExplorerView>> = opened(cx);
        if want.is_empty() {
            assert!(
                windows.is_empty(),
                "{}: nothing opens with no tick",
                fixture.name
            );
        } else {
            let ticked: LangSet = windows
                .first()
                .unwrap()
                .read_with(cx, |v: &ExplorerView, _: &App| v.langs.langs.ticked())
                .unwrap();
            assert_eq!(ticked, want, "{}", fixture.name);
        }
        close_all(cx);
    }
}

#[gpui::test]
fn with_no_box_ticked_open_does_nothing_and_the_page_says_why(cx: &mut TestAppContext) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = FIXTURES[0].write(base.path()).unwrap();
    let (page, mut vcx): (WindowHandle<StartPage>, VisualTestContext) = page_for(cx, vec![root]);
    click_tick(&mut vcx, Lang::CSharp);
    assert_eq!(page_ticks(page, &vcx), LangSet::EMPTY);
    vcx.simulate_keystrokes("enter");
    vcx.run_until_parked();
    assert_eq!(opened(cx).len(), 0);
    assert!(
        page.read_with(&vcx, |_: &StartPage, _: &App| ()).is_ok(),
        "the page stays"
    );
    close_all(cx);
}

#[gpui::test]
fn the_saved_ticks_come_back_when_the_workspace_is_picked_again(cx: &mut TestAppContext) {
    let store: tempfile::TempDir = tempfile::tempdir().unwrap();
    test_store(Some(store.path().to_path_buf()));
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = FIXTURES[6].write(base.path()).unwrap();
    let (_, mut vcx): (WindowHandle<StartPage>, VisualTestContext) =
        page_for(cx, vec![root.clone()]);
    click_tick(&mut vcx, Lang::CSharp);
    click_tick(&mut vcx, Lang::Python);
    vcx.simulate_keystrokes("enter");
    vcx.run_until_parked();
    close_all(cx);
    let (again, vcx): (WindowHandle<StartPage>, VisualTestContext) = page_for(cx, vec![root]);
    let restored: LangSet = page_ticks(again, &vcx);
    close_all(cx);
    test_store(None);
    assert_eq!(restored, LangSet::of(&[Lang::Rust]));
}

#[gpui::test]
fn file_languages_reopens_the_window_with_the_new_ticks(cx: &mut TestAppContext) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = FIXTURES[3].write(base.path()).unwrap();
    let first: WindowHandle<ExplorerView> = open_with(cx, &root, LangSet::of(&[Lang::CSharp]));
    let mut wcx: VisualTestContext = VisualTestContext::from_window(first.into(), cx);
    wcx.dispatch_action(super::super::langs::ChangeLanguages);
    wcx.run_until_parked();
    let page: WindowHandle<StartPage> = cx
        .windows()
        .into_iter()
        .find_map(|w: AnyWindowHandle| w.downcast::<StartPage>())
        .unwrap();
    let mut pcx: VisualTestContext = VisualTestContext::from_window(page.into(), cx);
    assert_eq!(page_ticks(page, &pcx), LangSet::of(&[Lang::CSharp]));
    click_tick(&mut pcx, Lang::Rust);
    pcx.simulate_keystrokes("enter");
    pcx.run_until_parked();
    let windows: Vec<WindowHandle<ExplorerView>> = opened(cx);
    assert_eq!(windows.len(), 1, "the old window closed");
    let langs: bb_lang::Langs = windows
        .first()
        .unwrap()
        .read_with(cx, |v: &ExplorerView, _: &App| v.langs.langs)
        .unwrap();
    assert_eq!(langs.active(), LangSet::of(&[Lang::CSharp, Lang::Rust]));
    close_all(cx);
}
