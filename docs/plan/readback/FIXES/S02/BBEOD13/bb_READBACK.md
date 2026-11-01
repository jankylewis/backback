# BBEOD13 readback, part 1: Open, recent, Go to File

Date 2026-11-02, by backback_mm. Release `backback` from `bbeod-13-open`,
`BACKBACK_STATE_DIR` pointed at a scratch folder, harness copy.

| Check | How | Result |
|---|---|---|
| Finder/Dock launch (BB-BUG-006) | started in `/`, no argument, recent = harness | Explorer shows the harness, not `/` (`FINDER_LAUNCH.png`) |
| Recent list | after the launch | file is `backback recent v1` + the harness path |
| Go to File box | Cmd+Shift+O, typed `greets` | box lists `Core/Greeter.cs`, `_NUnitApex/GreeterSpec.cs` (`GO_TO_FILE.png`) |
| Go to File opens | Cmd+Shift+O `spec`, Down, Enter | tab `Greeter.cs` -> `OutcomeSpec.cs` (state log) |
| Cmd+O | `cmd-o` through the window | app windows 1 -> 2, the new one titled "Open" |

Not covered: choosing a path in the macOS open panel (a system dialog, not
driven here); the code path after it is the one the Finder launch uses.

## Part 2: Back/Forward, Go to Symbol, Build (2026-11-02)

Release `backback` from `bbeod-13-build`, harness copy (`PART2_STATE_LOG.txt`).

| Check | How | Result |
|---|---|---|
| F12 | click `Greeter` on line 13, F12 | `Greeter.cs` 3:21 (the class) |
| Back | Cmd+[ | `GreeterSpec.cs` 13:11 |
| Forward | Cmd+] | `Greeter.cs` 3:21 |
| Go to Symbol | Cmd+F12, `hello`, Enter | line 5 (`Hello`) |
| Build | writable copy with `int broken = notDeclared;`, Cmd+Shift+B | "Build failed: 1 error, 0 warnings", one clickable place |

Found while reading back: the first Build never finished (BB-PRC-002): a
process spawned from a background-QoS thread is throttled on macOS; the
build now starts on the UI thread and only waits in the background. A build
with `--no-write` is refused (it would write bin/ and obj/).
