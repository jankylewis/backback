# BB-BUG-093 — A C# repo with .py files started pyright with no choice

id: BB-BUG-093
family: UIX
status: FIXED
date: 2026-11-20
signature: `Python: starting pyright` (or a `pyright-langserver --stdio` child) in a C# window whose Python box nobody ticked
source: headmate ruling 2026-11-20 (BBEOD41); `bb_plan.rs:73` `default_ticks`, `bb_early.rs:67` `finish`, `ticks.rs:33` `saved_or_found` at 7fb7aaf
observed: a C# repo holding `.py` helper scripts opened with C# and Python ticked; pyright ran, Python showed in the status bar and Tests panel
cause: BB-UIX-026
links: fix=yes lesson=no evidence=no toolgap=none

## Steps

Open a folder holding a C# solution and a `pyproject.toml` with `.py` files, never ticked before (command line, untouched Open..., or a RECENT row); read the ticks, the status bar and `ps` under the app.

Expected (ruling): "if only tick c# -> ignore everything about python in the opened repo"; Python is never on unless the user ticked it.

Actual: C# and Python ticked; pyright started at open; the found set was saved as if the user picked it, so later opens kept Python.
