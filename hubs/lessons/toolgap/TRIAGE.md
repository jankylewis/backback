# Triage — tool gaps (BB-TG)

One row per tool gap, by id; the entry point is `hubs/failures/TRIAGE.md`.
Grep the signature in your log.

| Id | Signature | Status | First check |
|---|---|---|---|
| BB-TG-015 | `bb_diag_state version=None` | FIXED | Is the binary older than the runner flush (`flush_as_a_paint`)? Then ask a question before reading `problems` |
| BB-TG-016 | `a readback hangs at 0 % CPU` | FIXED | Did the launch open a file? Binary and folder launches start the steps since BB-TG-016 |
| BB-TG-017 | `progress.last() (2, 3) != (3, 3)` | FIXED | Read the failed list in the message: which project did Roslyn fail to load? |
| BB-TG-018 | `a readback gets slower each round` | OBSERVED | Is `ui_work` still small? Then the runner's timers are throttled (likely App Nap), not the app |
| BB-TG-019 | `a runner press outside a popup did nothing` | OBSERVED | Did the readback window draw a frame after the popup opened? |
| BB-TG-020 | `where is X defined / what does this crate do` | FIXED | Run `bb map` / `bb map --find X` before opening files |
| BB-TG-021 | `test memory peak above the runner's RAM` | FIXED | Is it a sum of RSS? Read `vm_stat` in-use pages and swap |
| BB-TG-022 | `FolderEntries` or `Marker` on `.claude/skills` or `.agents/skills` | FIXED | Is the walk skipping folder rules for the root agent trees? |
| BB-TG-023 | a gate takes many minutes for a small change | FIXED | `gate.sh --plan`: is the scope right? `--full` only when unsure |
