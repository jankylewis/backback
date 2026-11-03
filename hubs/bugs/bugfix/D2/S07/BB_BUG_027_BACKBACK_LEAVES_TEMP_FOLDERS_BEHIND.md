# BB-BUG-027 — fix

id: BB-BUG-027
family: PRC
status: FIXED
source: `bb_crates/bb_core/process/src/temp.rs`, `bb_crates/shell/desktop/src/intel/bb_worker/mod.rs` (`prepare`), `docs/release/scripts/gate.sh`

## The change

`bb_process::OwnedTempDir`: a folder named `<prefix>-<pid>[-<suffix>]`,
removed on drop and at app quit (`bb_process::shutdown`). Its create
sweeps first: the prefix's folders whose pid no process has (`ps -p`) go.
The app's intel folder is owned for the life of its loop. The testkit
sweeps its family once per process. `bb sweep` sweeps every family
(`FAMILIES`); the gate runs it after the tests, and deletes its own log
when green.

## Proof

Unit tests: a dead owner's folder swept, a live one (pid 1) kept, drop
and quit remove. The first `bb sweep`: 3156 folders removed; 17 kept
whose pid is live (the headmate's open app, and pids reused since).
`leak_probe.py` (5 test runs on the branch build): RSS 98-109 MB, 65 open
files, 7-8 threads, 2 children, all flat; after quit the app's
`backback-intel-<pid>` was removed and of 40 descendants only
`VBCSCompiler` (dotnet's shared compiler server) lived on.
