# BBEOD06-FIX readback and lessons, 2026-11-01

Fixes for the BBEOD06-VERIFY findings on `bbeod-06-gui` (backback_1).
Each fix had a test that failed on the old code first.

## Readback (real window, fixture only)

- Debug app on `_Harness` with `BB_HARNESS_DEMO=1`. `RUNNING_FIX.png`: green
  `● running`, Greet result `hi Ada` drawn as rows. `STOPPED_FIX.png`: grey
  `○ stopped`. Window-ID captures (`screencapture -l`), inspected.
- During the run the app had one child (`dotnet`). After Stop it had none,
  and no `McpServer.dll` process was left.

## Findings and fixes

| Finding | Red on old code | Fix |
|---|---|---|
| B3 grey dot | badge test: all states `text_dim` | state badge tokens; light `ok` is now green |
| B4 schema form | `opt.must: required`, int enum `outside enum` | schema kinds: integer, enum values, arrays, JSON, nullable, optional parents |
| B5 server ping | `mismatched envelope for initialize` | reader task answers server requests at any time |
| B6 10 MB result | first paint 9.61 s, redraw 1.78 s | rows built off the UI thread, `uniform_list` |
| B7 error text | `os error 2`, `line 1 column 1` | no `transparent` errors; each names its cause |
| B8 Stop 2 s | `signal: 15 (SIGTERM)` after fallback | stdin is really closed |
| F1 flake | build warning reached stdout: `NotJson` | build, then run the built dll |

## Bug families (name, shape, guard)

1. **Close assumed, not done.** tokio `ChildStdin::shutdown()` is a no-op on
   unix (`process/unix/mod.rs:294`). The server never saw EOF; every Stop
   waited 2 s, then sent SIGTERM. Guard: stdin lives in `Option` and Stop drops
   it. Swept: only MCP used `shutdown()` on a pipe; `bb_lsp` uses LSP shutdown.
2. **Tool output on the protocol channel.** `dotnet run` writes build warnings
   to stdout when it builds. One warning ends the handshake. Guard: `Session::start`
   runs `dotnet build -t:Build -getProperty:TargetPath` (stdout is the path
   only), then `dotnet <dll>`. Builds are serialised in the process.
3. **Error without its cause.** `#[error(transparent)]` showed bare OS and
   serde text. Guard: `McpError` has no `transparent` or `#[from]` variant, so
   every construction site must name the operation. A test checks all variants.
4. **Wait helper hides the cause.** The window test waited 20 s for Running
   and ignored `Crashed`, so a crash read as a timeout. Guard: both wait helpers
   fail at once on `Crashed` and print status, error and the last 5 log lines.
5. **Unbounded text drawn whole.** One 10 MB string was one text element; the
   log list built every line. Guard: `RowText` (160-char rows) and
   `uniform_list` for the result and the log.

## F1 root cause status

F1 did not reproduce in 23 runs of the old code: 10 harness runs, 3 with cold
fixture bin/obj, and 10 full-binary runs at load1 up to 29.8. The failing run
timed out in the Running wait, about 1 s into the run. The confirmed mechanism
that fits it is family 2: build output on stdout gives `Crashed`, and family 4
hid that as a timeout. The exact line printed in that run was not observed.
After the fix: 20/20 consecutive full desktop-binary runs at 2 threads,
load1 10.4-22.2, run 1 on cold fixture bin/obj. Gate: 153 passed, 0 failed.

## Open (not in this brief)

- B1: a server that ignores stdin EOF can outlive a SIGTERM-ed `bb mcp`.
- B2: Stop during a hung Start still waits for the 30 s initialize deadline.
