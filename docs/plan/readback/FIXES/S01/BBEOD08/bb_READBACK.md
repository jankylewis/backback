# BBEOD08-REAP readback: quitting leaves no process behind

Date 2026-11-02, by backback_mm, after rebasing REAP on `master`.

## Setup

Release `backback` from `bbeod-08-reap`, `--no-write`, on a copy of the
`_Harness` fixture opened at `_NUnitApex/GreeterSpec.cs`. After Roslyn was
ready, the app's whole process tree was recorded (`pgrep -P`, recursive),
then the app was ended three ways and every recorded process checked.

## Result (`QUIT_CHECK.txt`)

| Exit | App | `roslyn-language-server` | `Microsoft.CodeAnalysis.LanguageServer` |
|---|---|---|---|
| Quit (`cx.quit`, the menu's path) | exited | gone | gone |
| Cmd+Q through the window | exited | gone | gone |
| SIGTERM | exited | gone | gone |

## Not covered

- A test run and a debug session from the Test Explorer (BB-BUG-004 is
  still OPEN); the testkit and DAP owners are covered by red-first tests.
- `BB_KEYS` starts only once a file tab is open; a run on the bare `.slnx`
  never ran its script (tooling limit, noted for readbacks).
