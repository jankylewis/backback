---
name: cda-backback-ship
description: "Take a backback change from a green worktree to merged, cleaned up and installed, with the version checked."
metadata:
  source: skillhub
---

# cda-backback-ship

Take a backback change from a green worktree to merged, cleaned up and installed, with the version checked.

- Audit, then the gate in the background from the worktree: `bash docs/release/scripts/gate.sh` (the changed crates and their users; `--plan` shows the scope, `--full` when unsure; CI runs the full suite); quote its counts (suites, passed, failed). Red: fix, record the failure in hubs (cca-backback-hubs), gate again. Gate again after every rebase.
- Commit as `jankylewis <jankylewis.se@gmail.com>` with the attribution line; push; `gh pr create -R jankylewis/backback --base master --body-file <file>` with GH_TOKEN from `gh auth token --user jankylewis`.
- Wait for CI in the background: merge only when `gh pr checks <n> --json bucket` is all `pass` with at least one check. A run 'not acquired by Runner' gets a rerun, never a merge over it.
- Merge with `gh pr merge <n> --merge`, then clean up only after MERGED: `st=$(gh pr view N --json state -q .state); [ "$st" = MERGED ] && git worktree remove <path> && git branch -d <branch>`, delete the remote branch, `git pull --ff-only` the primary, empty the scratch files and remove `$TMPDIR/bb_gate.*` (guard every rm path with `${VAR:?}`).
- Install only while the user's app is closed: wait on `pgrep -x backback`, then `env -u CARGO_TARGET_DIR bash docs/release/scripts/install.sh` from the primary checkout; tell the user to close the app.
- Verify: `/usr/libexec/PlistBuddy -c 'Print :BBCommit' /Applications/backback.app/Contents/Info.plist` equals master. Only then say 'installed'.
- Load with `agentfarm skillload <seat> cda-backback-ship`.
- Codex consumes this variant: run long steps (gate, CI wait, install) as separate commands and read their exit codes; never chain a commit after a grep.

## Guardrails

- Self-merge is granted on jankylewis/backback only; never `--force`, never `git branch -D`, never `agentfarm down`.
- Never claim fixed, merged or installed without the command output that shows it.
- Only stop processes this session started, one pid at a time; never `pkill`, never the user's running backback.
- Nothing is filed in ADO without the headmate's explicit approval.
