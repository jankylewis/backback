# BB-UIX-026 — An auto-detected language set acted as the user's choice

id: BB-UIX-026
family: UIX
status: FIXED
date: 2026-11-20
signature: a `langs` record the user never picked; Python ticked in a C# window after one open
source: `bb_plan.rs:73` `default_ticks`, `bb_early.rs:67` `finish` (`save_langs(root, found.found)`), `bb_view/mod.rs:116` `lang_mark`, `bb_window.rs:127` menu, `places.rs:224` and `find.rs:68` projects, `icons.rs:90` at 7fb7aaf
observed: BB-BUG-093; the real store held 35 v1 records, 5 with Python (1 Python only), none told apart from picks
links: fix=yes lesson=no evidence=no toolgap=none

## What it is

Detection answered two questions as one: "what does the folder hold?" and "what did the user tick?". The found set became the ticks, then was saved as a choice and read back as one. Places that asked only the file's language (tab mark, Python Check, Find in Files projects, Python project icon) showed Python even when `Langs` said it was off.

## How to confirm it

Open a C# + Python folder with no saved pick; `ps -o pid,ppid,command` for `pyright` under the app; read the `langs` file: a new line appeared with no tick clicked.
