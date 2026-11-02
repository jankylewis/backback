# BBEOD18 readback: clickable stack traces in test results

Date 2026-11-03, by backback_mm. Branch `bbeod-18-stack-links`.

## What

The Tests panel's failure detail (and the raw run output) is drawn line by
line: a stack frame with a source (`at N.C.M() in <file>:line <n>`) is a
link (accent colour, underline on hover) that opens its own call site;
framework frames without source stay plain. The path is read after the
method's `) in `, so a folder named `x in y` stays whole; deterministic
build paths (`/_/...`) map to the opened root; relative paths too; a file
not on this machine is said in the status bar.

## Result

| Check | Evidence |
|---|---|
| Frame shapes: plain, generic, lambda, async `MoveNext`, a path with ` in `, Windows `C:\`, `/_/` | `every_dotnet_frame_shape_gives_its_file_and_line` |
| No source, message text, bad or zero line | `frames_without_source_and_other_text_give_nothing` |
| Click frame 3 opens `B.cs` line 7; framework frame is plain; `/_/B.cs` opens `B.cs` line 2; a missing file reports | `each_stack_frame_opens_its_own_call_site` (window, real clicks on drawn rows) |
| A real NUnit failure still opens its spec at the failing line | `test_panel_lists_runs_a_class_and_opens_a_failure` |

With the row's `stop_propagation` removed, the window test fails (the
panel's click on the first frame would win).

## Not covered

A click in the running app: the screen was locked (BB-TG-010); the window
tests click the drawn rows through the same handlers.
