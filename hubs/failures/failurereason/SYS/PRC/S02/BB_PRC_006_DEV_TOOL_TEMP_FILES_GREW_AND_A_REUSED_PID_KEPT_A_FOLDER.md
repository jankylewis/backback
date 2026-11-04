# BB-PRC-006 — Dev-tool temp files grew; a reused pid kept a folder

id: BB-PRC-006
family: PRC
status: FIXED
date: 2026-11-04
signature: `backback-roslyn-logs` and `backback-obj` in the temp folder grow with every `bb intel open` / `bb probe` run
source: `bb_crates/shell/bb_cli/src/bb_commands/bb_intel/` (`bb_open.rs` `scratch_dir`, `create_log`); `bb_core/process/src/temp.rs` (`alive`)
observed: final leftover check after the night run (2026-11-04)
links: fix=yes lesson=no evidence=yes toolgap=none

## What it is

Three leaks, all outside the installed app (the app left nothing):

1. `bb intel open` and `bb probe` wrote one Roslyn log per run into
   `backback-roslyn-logs/` and never removed one: 1,435 logs, 17 MB.
2. Their scratch build folder `backback-obj/<solution>-<pid>` was removed
   only at the end of a good `bb intel open` run; an error return skipped
   it, and `bb probe` never removed it: 24 folders left.
3. The sweep took "pid alive" as "owner alive". macOS reuses pids: a
   `backback-intel-98906` folder from 2026-11-03 was kept because pid
   98906 was now another app.

## How to confirm it

`ls $TMPDIR | grep backback`, then `ps -o etime=,command= -p <pid>` for
each folder's pid: a process younger than its folder is not the owner.
