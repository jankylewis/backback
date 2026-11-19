# BB-UIX-010 — The key map lacked VS Code's line commands

id: BB-UIX-010
family: UIX
status: FIXED
date: 2026-11-03
signature: a line-command key does nothing
source: `bb_crates/shell/desktop/src/editor/input/bb_keys.rs` (`document_command`), `bb_crates/bb_core/bb_text/src/document/edit/`
observed: sweep 4 on the installed app at `38baa8b` (BB-BUG-020, BB-BUG-021)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The Cmd table maps a key to a text-model command and drops every other
key. The text model had line-wise indent and outdent behind Tab and
Shift+Tab, but no command for a whole line: no delete, no new line below
or above, no move, no copy, no select. Nine keys of VS Code's line
commands did nothing; after Cmd+Enter or Cmd+L the next key typed at the
cursor.

## How it shows

A sweep that types a key then saves finds the file unchanged, or the next
key in the middle of the line.

## How to confirm it

Run `sweep04_run.py`'s cases (Words.cs, no solution) on the build; any
UNCHANGED verdict is a key this map drops.

Status OPEN: Cmd+Shift+K, Cmd+(Shift+)Enter (BB-BUG-020) and Cmd+L are
in; moving and copying lines are not, and Cmd+[ / Cmd+] are Back/Forward
here (BB-BUG-021).
