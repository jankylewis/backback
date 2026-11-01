# BBEOD10 readback: Home, End, Cmd+arrows and Cmd+M

Date 2026-11-01, by backback_mm. Bugs BB-BUG-001 (Cmd+M) and BB-BUG-007
(Home/End).

## Setup

- App: release `backback` from branch `bbeod-10-keys`, `--no-write`, on a
  copy of `CapturedOutboundRequest.cs`.
- Input through the window's own dispatch (`BB_KEYS`): `click:3:33`, then
  `end`, `home`, `cmd-left`, `cmd-down`, `cmd-up`, `cmd-m`. After each key a
  `state` step prints the cursor (`STATE_LOG.txt`). After each `mark:` the
  window list says whether the app window is on screen (`WINDOW_ON_SCREEN.txt`).

## Result

| Key | Cursor (line, col) | Expected |
|---|---|---|
| click 3:33 | 3, 33 | the clicked column |
| End | 3, 46 | after `(`, the line has 45 chars |
| Home | 3, 1 | first non-blank (no indent) |
| Cmd+Left | 3, 1 | line start |
| Cmd+Down | 5, 1 | document end (file ends with a newline) |
| Cmd+Up | 1, 1 | document start |
| Cmd+M | window on screen: none | minimized |

Before Cmd+M the window was on screen at every mark; after it, the window
list no longer shows it: macOS minimized it to the Dock.

## Not covered

- Window captures were stale in this run, most likely because another window covered
  backback, and `screencapture -l` of a covered GPUI window returns its last
  frame (BB-TG-005). The caret painting is proven by the BBEOD12 pixel diff;
  this run proves the key handling through the model and the window list.
- Cmd+H (hide) was not run here: hiding the app ends the capture session.
  It is bound and covered by `cmd_m_minimizes_cmd_h_hides_and_cmd_alt_h_hides_others`.
- A first run's `click` step did not land; the rerun with a 3 s settle did.
