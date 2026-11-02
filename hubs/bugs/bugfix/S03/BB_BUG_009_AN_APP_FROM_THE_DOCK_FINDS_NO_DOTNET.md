# BB-BUG-009 — fix

id: BB-BUG-009
family: SHIP
status: FIXED
source: `bb_crates/bb_core/process/src/shell_env.rs`, called first in `launch()`

## The change

As VS Code does ("resolve shell environment"): when the app has no `TERM`
(not started from a terminal), it runs the login shell once, `$SHELL -l -i
-c`, within 5 s, reads its `PATH` between markers (banners ignored) and
adopts it before any thread or child starts. If the shell gives nothing, the
.NET and Homebrew folders are added instead.

## Proof

Tests: no-terminal rule, marker parsing with banners, fallback folders, a
real `/bin/sh` login gives its `PATH`, a missing shell gives none. Readback:
the bundle launched with `env -i` and launchd's `PATH`: C# server Ready with
no failed project; Cmd+Shift+B "Build succeeded: 0 warnings".
