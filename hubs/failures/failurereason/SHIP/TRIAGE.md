# Triage — SHIP failures (BLD, CIP, GAT)

One row per failure reason of this layer, by family and id; the entry point is
`hubs/failures/TRIAGE.md`. Grep the signature in your log.

| Id | Signature | Status | First check |
|---|---|---|---|
| BB-BLD-001 | `app built before the fix is still running` | FIXED | `ps -o lstart` and binary mtime vs merge time |
| BB-BLD-002 | `.app opens no window` | FIXED | `CFBundleExecutable` is the GUI binary; run `verify_bundle.sh` |
| BB-BLD-003 | `still broken after the fix merged` | FIXED | `BBCommit` of the running app (status bar) holds the fix? |
| BB-BLD-004 | `a dark tile behind the Dock icon` | FIXED | Is the icon set at launch (`bb_dock::set_icon`)? Is `dock.png` transparent? |
| BB-CIP-001 | `roslyn-language-server not found; set BACKBACK_ROSLYN_LS` | FIXED | `bb_ci.yml` installs the pinned Roslyn and netcoredbg |
| BB-CIP-002 | `gate:queued` | FIXED | `runs-on` is a label a hosted image serves (`macos-latest`) |
| BB-CIP-003 | `error[yanked]: detected yanked crate` | FIXED | Ask crates.io for the version list; `cargo update -p <crate>` |
| BB-CIP-004 | `not acquired by Runner of type hosted` | FIXED | Rerun it; merge only when every `--json bucket` is `pass` |
| BB-CIP-005 | `OPEN` then the worktree is gone | FIXED | Is cleanup gated on the state value MERGED? |
| BB-CIP-006 | `RUSTSEC-2026-0592` (url 2.5.8) in the Dependency audit | OBSERVED | `cargo deny check advisories`; `cargo update -p <crate> --precise <fixed>` |
| BB-GAT-001 | `incremental_parse_equals_a_fresh_parse ... FAILED` | FIXED | Is the text broken? Then only the extent must match |
| BB-GAT-002 | `fixture build failed` ... `being used by another process` | FIXED | Do parallel test processes share a fixture folder? Use `OwnedTempDir` |
| BB-GAT-003 | `discovered=0 loading=1` in a native window test | FIXED | Does the test sleep a fixed time? Use `wait:tests` |
| BB-GAT-004 | a menu test counts one item too few | FIXED | Did a menu or key change skip the launch tests? Update the count and name the item |
| BB-GAT-005 | `the search did not end` in a window test | FIXED | Does the test wait for a state only a later step starts? Read it before that step |
| BB-GAT-006 | `clippy rc=101` on a tree whose tests passed | FIXED | Was the worktree edited while its gate ran? Rerun on the still tree |
| BB-GAT-007 | `no method named` ... `tests: none run` after audit fixes | FIXED | Was the tree built after the last edit? Build the tests before the gate |
| BB-GAT-008 | `wildcard match will also match any future added variants` in a test | FIXED | Did clippy run on the touched crates before the gate? |
| BB-GAT-009 | `got Refused("Set Next Statement refused: adapter does not support` in CI only | FIXED | Is CI on the stock adapter? Run the test with `BACKBACK_NETCOREDBG` on the stock copy |
| BB-GAT-010 | a test run idle at 0 % CPU for minutes; or `waited <time> for <what>` | FIXED | Which wait? Does it arm the watchdog? Run the test alone under an alarm |
| BB-GAT-011 | `bb map` item on the wrong line; `... is_on_the_map ... missing: pub fn` | FIXED | Does the file continue a string with `\\` + newline? Is `skip_escape` used? |
| BB-GAT-012 | a parsed tool message listed twice on CI, once locally (`left: 2` `right: 1`) | OBSERVED | Does the tool build the same file in two units? Count with `-j2` |
| BB-GAT-013 | a variant test red on the first machine with the tool | FIXED | Run with `BACKBACK_REQUIRE_TOOLS=1`; compare canonical paths |
| BB-GAT-014 | `Detected activity on thread ... not deterministic` | FIXED | Does a foreign thread wake a gpui task? Forward with `try_recv` |
| BB-GAT-015 | `Timeout { method: "initialize"` in a test with a short deadline | FIXED | Does the config's `request_timeout` (the start's deadline) equal the request's? |
| BB-GAT-016 | a variant test green in CI in milliseconds | OBSERVED | Does CI install the tool and set `BACKBACK_REQUIRE_TOOLS=1`? Read the `before:` lines |
| BB-GAT-017 | `items after a test module`; `Iterator::last` on a `DoubleEndedIterator`; map check `missing: pub(crate) fn` | FIXED | Is a test-only helper a named `#[cfg(test)]` module mid-file? Run clippy `--all-targets` and `-p bb-kmap --test views` |
| BB-GAT-018 | `the window waited for the walk` on CI; nextest `LEAK` after a real-server test | FIXED | `editor_open_ms` vs the hold cap; does the test `wait_ended`? `ps -o ppid` of BuildHost/servers (1 = orphan) |
| BB-GAT-019 | `left: ["<pid> .../deps/backback-<hash> --exact ...` (the test's own row) | FIXED | Does the probe match on the command line? nextest puts the test name there; exclude the test's own pid tree |
| BB-GAT-020 | `servers missing ["sleep 600"]` with `printed true` | FIXED | Does the script end backback on its own timer? Hold it at `wait:go` until `up()` saw every process |
