# BBEOD23 readback: the start page (2026-11-03)

Branch build of `bbeod-23-start-page`. A scratch recent store
(`BACKBACK_STATE_DIR`) holds `Alpha`, a folder gone from disk (`Gone`) and
`Beta`, newest first; the real recent list is not touched (its mtime is the
same before and after). The page's runner prints `bb_start_state`.

| Case | Launch, keys | Result |
|---|---|---|
| R1 a Dock-like launch | working folder `/`, no argument; `sleep:2000 state` | PASS: `bb_start_state shown=true selected=0 entries=["Alpha …/Alpha", "Beta …/Beta", "Open..."]`; `Gone` left out |
| R1 Down, Enter | `down enter sleep:2500 state` | PASS: the page closes; the workspace window shows `root="…/Beta"` and `bb_start_state shown=false` |
| R2 a file argument | `…/Alpha/Note.cs`, `sleep:1500 state` | PASS: `root="…/Alpha"`, tab `Note.cs`, no page |

Tests: `the_entries_keep_the_recent_folders_on_disk_newest_first_then_open`,
`a_path_under_home_is_shown_with_a_tilde`,
`down_and_enter_open_the_second_entry_and_close_the_page` (window: the page
opens the second entry in its own workspace window and closes), and
`a_finder_launch_shows_the_start_page_and_never_opens_the_disk_root`
(BB-BUG-006 kept: a launch from `/` never opens `/`).

## Verdict

PASS. File > Start Page... opens the page again from any window.
