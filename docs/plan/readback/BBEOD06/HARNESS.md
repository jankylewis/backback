# BBEOD06-B Harness readback, 2026-11-01

- Opened the fixture `_Harness` in the desktop app. The panel listed one executable MCP project, `_McpServer`, as stopped. `HARNESS_STOPPED.png` is a `screencapture -l` capture of the running window.
- In the same desktop build, the fixture-only debug drive started `_McpServer`, selected Greet, entered `Ada`, and called it. `HARNESS_GREET.png` shows running status, the tool list, form, and JSON text `hi Ada`. It is a window-ID capture, inspected after capture.
- The in-process GPUI test drove a simulated mouse click on Start, dispatched a keystroke to the form, then stopped the fixture. The model test covered Greet, a crash during Hold, and no remaining process after Stop.
- The MCP engine test observed the timestamped startup stderr line. The Log tab, search, pause, and clear are implemented; no window-ID screenshot of that tab was captured.
- No real MCP server was started. The prior engine readback listed MbTx 0, NmcTx 1, and SI 1 MCP projects.

## Lesson

The panel must wait for the actor's stop event before it labels a process stopped. A fixture test failed on this state race before the `stopping` state fixed it.

The engine merge moved the fixture. Three served fixture tests failed at process start because test and example paths stayed stale. Both path consumers now use the desktop fixture path; three fixture tests and the latency example passed after the fix.
