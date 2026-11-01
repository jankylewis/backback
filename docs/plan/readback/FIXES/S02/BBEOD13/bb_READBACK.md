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
