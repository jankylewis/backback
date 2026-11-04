//! Notes from the session thread applied to the window: a stop opens its
//! line and fills the Debug panel; a call-stack pick shows its source and
//! variables; the paused line is what the editor highlights.

use std::path::PathBuf;

use gpui::{Context, Window};

use crate::run::bb_debug::session::control::{Live, Phase};
use crate::run::bb_debug::session::{Command, Frame, Note, Var};
use crate::ExplorerView;

/// Program output kept for the panel, in chars.
const OUTPUT_CAP: usize = 20_000;

/// One note from the session thread.
pub(crate) fn apply(
    view: &mut ExplorerView,
    note: Note,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let Some(live): Option<&mut Live> = view.debug.live.as_mut() else {
        return;
    };
    let label: String = live.label.clone();
    match note {
        Note::Building => view.status = Some(format!("Building {label}...")),
        Note::Running => {
            live.phase = Phase::Running;
            view.status = Some(format!("Running {label}"));
        }
        Note::Stopped {
            reason,
            exception,
            frames,
            vars,
        } => stopped(view, (reason, exception), (frames, vars), window, cx),
        Note::Variables { frame, vars } => variables(live, frame, vars),
        Note::Output(text) => keep_output(&mut live.output, &text),
        Note::CanSetNext(can) => live.can_set_next = can,
        Note::Refused(why) => view.status = Some(why),
        Note::Ended(why) => {
            view.debug.live = None;
            view.status = Some(format!("Debugging ended: {why}"));
        }
        Note::Failed(why) => {
            view.debug.live = None;
            view.report(&format!("Debug failed: {why}"));
        }
    }
}

/// A stop: show the first frame with source and say why and where; on an
/// exception, which one.
fn stopped(
    view: &mut ExplorerView,
    (reason, exception): (String, Option<String>),
    (frames, vars): (Vec<Frame>, Vec<Var>),
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let name: String = frames
        .first()
        .map(|f: &Frame| f.name.clone())
        .unwrap_or_default();
    let shown: usize = frames
        .iter()
        .position(|f: &Frame| f.path.is_some())
        .unwrap_or(0);
    view.status = Some(match exception.as_deref() {
        Some(what) => format!("Paused on {what} in {name}"),
        None => format!("Paused ({reason}) in {name}"),
    });
    if let Some(live) = view.debug.live.as_mut() {
        live.hidden = false;
        live.phase = Phase::Stopped {
            frames,
            selected: shown,
            vars,
            exception,
        };
    }
    open_selected(view, window, cx);
}

/// A picked frame's variables, kept only if that frame is still picked.
fn variables(live: &mut Live, frame: i64, vars: Vec<crate::run::bb_debug::session::Var>) {
    if let Phase::Stopped {
        frames,
        selected,
        vars: shown,
        ..
    } = &mut live.phase
    {
        if frames.get(*selected).is_some_and(|f: &Frame| f.id == frame) {
            *shown = vars;
        }
    }
}

fn keep_output(output: &mut String, text: &str) {
    output.push_str(text);
    let extra: usize = output.chars().count().saturating_sub(OUTPUT_CAP);
    if extra > 0 {
        *output = output.chars().skip(extra).collect();
    }
}

/// A call-stack row was clicked: show its source and its variables.
pub(crate) fn select_frame(
    view: &mut ExplorerView,
    index: usize,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let Some(live): Option<&mut Live> = view.debug.live.as_mut() else {
        return;
    };
    let (frames, selected): (&mut Vec<Frame>, &mut usize) = match &mut live.phase {
        Phase::Stopped {
            frames, selected, ..
        } => (frames, selected),
        Phase::Building | Phase::Running => return,
    };
    let Some(frame): Option<&Frame> = frames.get(index).filter(|f: &&Frame| f.path.is_some())
    else {
        return;
    };
    let id: i64 = frame.id;
    *selected = index;
    live.send(Command::Frame(id));
    open_selected(view, window, cx);
}

/// Open the picked frame's file at its line.
fn open_selected(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    let Some((path, line)): Option<(PathBuf, usize)> = stopped_at(view) else {
        return;
    };
    let at: bb_text::LspPoint = bb_text::LspPoint {
        line: u32::try_from(line).unwrap_or(u32::MAX),
        character: 0,
    };
    crate::intel::glue::goto_point(view, &path, at, window, cx);
}

/// The paused frame's file and 0-based line, for the editor's highlight.
pub(crate) fn stopped_at(view: &ExplorerView) -> Option<(PathBuf, usize)> {
    let (frames, selected): (&Vec<Frame>, &usize) = match &view.debug.live.as_ref()?.phase {
        Phase::Stopped {
            frames, selected, ..
        } => (frames, selected),
        Phase::Building | Phase::Running => return None,
    };
    let frame: &Frame = frames.get(*selected)?;
    Some((frame.path.clone()?, frame.line))
}

/// One line for readback logs: the phase, frames as `name:line` (1-based)
/// and the variables.
pub(crate) fn describe(view: &ExplorerView) -> String {
    let Some(live): Option<&Live> = view.debug.live.as_ref() else {
        return String::from("session=none");
    };
    match &live.phase {
        Phase::Building => String::from("session=building"),
        Phase::Running => String::from("session=running"),
        Phase::Stopped {
            frames,
            selected,
            vars,
            exception,
        } => {
            let stack: Vec<String> = frames
                .iter()
                .map(|f: &Frame| format!("{}:{}", f.name, f.line.saturating_add(1)))
                .collect();
            let values: Vec<String> = vars
                .iter()
                .map(|v: &Var| format!("{}={}", v.name, v.value))
                .collect();
            format!(
                "session=paused selected={selected} exception={exception:?} frames={stack:?} vars={values:?}"
            )
        }
    }
}
