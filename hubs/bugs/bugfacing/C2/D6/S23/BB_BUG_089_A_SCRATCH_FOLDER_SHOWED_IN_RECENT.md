# BB-BUG-089 — A scratch folder showed in RECENT

id: BB-BUG-089
family: UIX
status: OBSERVED
date: 2026-11-19
signature: a `/private/tmp/...` folder in the start page's RECENT
source: BBEOD32 readback row 1a (c); `app/bb_launch/recent.rs` `add`
links: fix=no lesson=no evidence=no toolgap=none

## Steps

Open a scratch folder (an agent's readback fixture) once, then launch
with no argument.

## Verdict: not a bug

RECENT lists every workspace opened, newest first, as VS Code does; the
scratch folder was opened by an earlier session and still existed when
the shot was taken (BBEOD32 row 1a). Once a folder is gone it now shows
greyed with a remove button (BB-BUG-087's change, BB-FEAT-111); nothing
on disk is ever touched. Readback runs that must not leave entries set
`BACKBACK_STATE_DIR` to a scratch folder (`recent.rs` `state_dir`).
