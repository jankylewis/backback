# Triage — SYS failures (PRC, PRF, ENV)

One row per failure reason of this layer, by family and id; the entry point is
`hubs/failures/TRIAGE.md`. Grep the signature in your log.

| Id | Signature | Status | First check |
|---|---|---|---|
| BB-ENV-001 | `PATH=/usr/bin:/bin:/usr/sbin:/sbin` in `ps eww` of the app | FIXED | Started by launchd? The login shell's PATH must be adopted and installed tools found in their own folders |
| BB-ENV-002 | the machine runs hot during a gate | FIXED | Is `CARGO_BUILD_JOBS` 1 and `--test-threads=1`? Nothing else heavy running? |
| BB-ENV-003 | `rust-analyzer not installed` only from the Dock | FIXED | Did the login shell give a PATH? Is the tool's folder in `fallback_dirs`? |
| BB-PRC-001 | `process left after quit` | FIXED | Record the app's process tree before quit; check every pid after |
| BB-PRC-002 | `Building Harness.slnx... (never ends)` | FIXED | `ps -o stat` of the child: `SN` = spawned from a background thread |
| BB-PRC-003 | `SN roslyn-language-server` | OBSERVED | Is the app itself `N`? A zsh `&` launch is niced (BB-TG-006); rechecked on master 2026-11-20 |
| BB-PRC-004 | `a <prefix>-<pid> folder of a dead pid` | FIXED | Is the folder made through `OwnedTempDir` or swept by its family? |
| BB-PRC-005 | `a project load fails only with two workers` | FIXED | Do two owners in one process share a folder name? |
| BB-PRC-006 | `a temp folder kept although its owner is gone` | FIXED | Is the pid's process younger than the folder (`ps -o etime=`)? |
| BB-PRC-007 | `LEAK [ ... ] intel::bb_worker::tests` | FIXED | Does the test drain its server thread (`finish`/`end`) before returning? |
| BB-PRC-008 | `dotnet test` on a temp `Tests.csproj` with ppid 1 | FIXED | Did the session test drop its `Debugged` guard? Is the thread drained? |
| BB-PRC-009 | a `dotnet`, `MSBuild.dll`, `VBCSCompiler` or build step with ppid 1 and no backback running | FIXED | `ps -o pid,ppid,pgid,lstart`; is its group in a `children/` record? Is `guard_pid` alive while backback runs? |
| BB-PRC-010 | `VBCSCompiler -pipename:` with ppid 1 after a test run, its group no backback registered | FIXED | Did a test start `dotnet` without `bb_dotnet::no_servers::command`? `RoslynCommandLineLogFile`: `CompilerServer: server` = it used a server |
| BB-PRF-001 | `RSS 1,008 MB with the 20 MB file` | FIXED | Heap profile: who holds the bytes before changing code |
| BB-PRF-002 | `Go to File open freezes the window on a big folder` | FIXED | `BB_KEYS` UI work max at the feature cap (50,000 files) |
| BB-PRF-003 | `roslyn-language-server RSS grows past 2 GB` | FIXED | Is `DOTNET_gcServer` set in the app's environment (a user value wins)? |
| BB-PRF-004 | `UI work grows with line length` | FIXED | Does the slow path go through `Columns::shape` or `styled_line` (both capped)? |
| BB-PRF-005 | `two roslyn processes per window` | FIXED | Does `bb_direct::server_behind` find exactly one server in the tool store? |
| BB-PRF-006 | `Find in Files slow on a big folder` | FIXED | `find_bench <root> <query>`: walk_ms vs median; are both phases used? |
| BB-PRF-007 | `first_frame_ms grows with the folder on a launch with no saved languages` | FIXED | Is `walk_early` called before the app runs, and does `early_walk` match the root and ticks? |
| BB-PRF-008 | binary or `.app` size grew after adding a language | ACCEPTED | `size -m` of the new `libtree-sitter-*.a`; a linker map (`-Wl,-map`) by crate |
| BB-PRF-009 | `first_frame_ms` ~220 only on the first open of a big folder | FIXED | Does `build_window` take `Resolved::Later` (status "Detecting languages…")? Saved choice equal to base? Time `bb_lang::detect` on the tree; count its folders |
