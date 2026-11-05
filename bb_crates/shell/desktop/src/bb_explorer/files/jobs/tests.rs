//! BB-BUG-052: Copy / Paste, Copy Path and Open Terminal Here, on disk, in a
//! real shell, and through the window's right-click menu.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gpui::{TestAppContext, VisualTestContext};

use super::bb_ops::{copy_into, FileOpError};
use crate::files::tests::{click, right_click, status};
use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::ExplorerView;

#[test]
fn a_paste_takes_a_free_copy_name_and_copies_a_whole_folder() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let a: PathBuf = dir.path().join("A.cs");
    std::fs::write(&a, "class A {}").unwrap();
    assert_eq!(
        copy_into(&a, dir.path()).unwrap(),
        dir.path().join("A copy.cs")
    );
    assert_eq!(
        copy_into(&a, dir.path()).unwrap(),
        dir.path().join("A copy 2.cs")
    );
    let sub: PathBuf = dir.path().join("Sub");
    std::fs::create_dir_all(sub.join("deep")).unwrap();
    std::fs::write(sub.join("deep/x.txt"), "x").unwrap();
    std::os::unix::fs::symlink("deep/x.txt", sub.join("link")).unwrap();
    let other: PathBuf = dir.path().join("Other");
    std::fs::create_dir(&other).unwrap();
    let to: PathBuf = copy_into(&sub, &other).unwrap();
    assert_eq!(to, other.join("Sub"), "its own name where it is free");
    assert_eq!(std::fs::read_to_string(to.join("deep/x.txt")).unwrap(), "x");
    assert!(to
        .join("link")
        .symlink_metadata()
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(matches!(
        copy_into(&sub, &sub.join("deep")),
        Err(FileOpError::IntoItself(_))
    ));
    assert_eq!(
        std::fs::read_to_string(&a).unwrap(),
        "class A {}",
        "source kept"
    );
}

/// A real shell: the `cd` line reaches a folder whose name has a quote.
#[test]
fn the_cd_line_moves_a_real_shell_into_a_quoted_folder() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let odd: PathBuf = dir.path().canonicalize().unwrap().join("it's $here");
    std::fs::create_dir(&odd).unwrap();
    let (wake, _woken): (tokio::sync::mpsc::UnboundedSender<()>, _) =
        tokio::sync::mpsc::unbounded_channel();
    let session: crate::terminal::Session =
        crate::terminal::Session::start(dir.path(), 120, 12, wake).unwrap();
    session.input(b"half typed".to_vec());
    session.input(crate::terminal::cd_line(&odd));
    session.input(b"echo bb_at_$(basename \"$PWD\" | tr -d ' ')\r".to_vec());
    assert!(
        wait(|| screen_has(&session, "bb_at_it's$here")),
        "{:?}",
        session.screen().rows
    );
}

fn screen_has(session: &crate::terminal::Session, needle: &str) -> bool {
    session
        .screen()
        .rows
        .iter()
        .any(|r: &String| r.contains(needle))
}

fn wait(mut done: impl FnMut() -> bool) -> bool {
    let start: Instant = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        if done() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

#[gpui::test]
fn copy_then_paste_makes_a_copy_next_to_the_file(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    right_click(vcx, 1);
    assert!(
        vcx.debug_bounds("file-menu-paste").is_none(),
        "nothing copied yet"
    );
    click(vcx, "file-menu-copy");
    right_click(vcx, 0);
    assert!(
        vcx.debug_bounds("file-menu-copy").is_none(),
        "the root is not copied"
    );
    click(vcx, "file-menu-paste");
    let made: PathBuf = rig.root.join("A copy.cs");
    assert_eq!(
        std::fs::read_to_string(&made).unwrap_or_default(),
        "class A {}\n",
        "{}",
        status(&rig, vcx)
    );
    assert!(
        status(&rig, vcx).contains("Pasted A copy.cs"),
        "{}",
        status(&rig, vcx)
    );
}

#[gpui::test]
fn no_write_refuses_paste(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, true, true, "class A {}\n");
    right_click(vcx, 1);
    click(vcx, "file-menu-copy");
    right_click(vcx, 1);
    click(vcx, "file-menu-paste");
    assert!(!rig.root.join("A copy.cs").exists());
    assert!(
        status(&rig, vcx).contains("--no-write"),
        "{}",
        status(&rig, vcx)
    );
}

#[gpui::test]
fn copy_path_puts_the_full_path_on_the_clipboard(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    right_click(vcx, 1);
    click(vcx, "file-menu-copy-path");
    let text: Option<String> = vcx
        .read_from_clipboard()
        .and_then(|c: gpui::ClipboardItem| c.text());
    assert_eq!(text.as_deref(), Some(rig.file.to_string_lossy().as_ref()));
}

fn shell_has(rig: &Rig, vcx: &mut VisualTestContext, needle: &str) -> bool {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.terminal
            .session
            .as_ref()
            .is_some_and(|s: &crate::terminal::Session| screen_has(s, needle))
    })
}

fn type_in(rig: &Rig, vcx: &mut VisualTestContext, text: &str) {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.terminal
            .session
            .as_ref()
            .unwrap()
            .input(text.as_bytes().to_vec());
    });
}

#[gpui::test]
fn open_terminal_here_starts_the_shell_there_then_cds_it(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    let sub: PathBuf = rig.root.join("Sub");
    std::fs::create_dir(&sub).unwrap();
    right_click(vcx, 1);
    click(vcx, "file-menu-terminal");
    assert!(vcx.debug_bounds("terminal").is_some(), "the panel opened");
    let root_name: String = super::file_name(&rig.root);
    type_in(&rig, vcx, "echo bb_in_$(basename \"$PWD\")\r");
    let mut seen: bool = false;
    assert!(wait(|| {
        seen = seen || shell_has(&rig, vcx, &format!("bb_in_{root_name}"));
        seen
    }));
    rig.view.update_in(
        vcx,
        |v: &mut ExplorerView, window: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::terminal::open_in(v, Path::new(&sub), window, cx);
        },
    );
    type_in(&rig, vcx, "echo bb_now_$(basename \"$PWD\")\r");
    assert!(wait(|| shell_has(&rig, vcx, "bb_now_Sub")));
}
