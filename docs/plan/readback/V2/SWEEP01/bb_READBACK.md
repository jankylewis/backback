# Sweep readback 1: editor keys and the pane's edges (2026-11-03)

## Keys, on the installed app at `0b6406d`

`install.sh` build, launchd's PATH; each case on a fresh scratch copy of a
fixture, keys through the window's own dispatch, the saved bytes judged.
`Words.cs` is `class C\n{\n    int alpha = beta + gamma;\n    void M() { }\n}\n`.

| Case | Keys | Saved | Verdict |
|---|---|---|---|
| A mixed line endings | `goto:1:8 right x goto:3:15 right z cmd-s` on `class A\r\n{\n    int x = 1;\r\n    int y = 2;\n}\r\n` | `class A\r\nx{\n    int x = 1;\r\nz    int y = 2;\n}\r\n` | PASS: every line keeps its own ending |
| B no final newline | `goto:4:2 x cmd-s` on `class B\n{\n    int z = 3;\n}` | `...}x` | PASS: no newline added |
| C1 Cmd+Right, Cmd+Left | `goto:3:9 cmd-right x cmd-left y cmd-s` | `    yint alpha = beta + gamma;x` | PASS: line end; Home to the indent |
| C2 Cmd+Backspace | `goto:3:17 cmd-backspace cmd-s` | unchanged | **BB-BUG-014**: nothing deleted |
| D undo, redo across word deletes | `goto:3:29 alt-backspace alt-backspace cmd-z cmd-z cmd-shift-z cmd-s` | `    int alpha = beta + ;` | PASS: one undo stop per word delete |
| E1 Tab with a selection | `goto:3:1 shift-down tab cmd-s` | `\t    void M() { }` (line 3 gone) | **BB-BUG-015**: the selection replaced by a tab |
| E2 Shift+Tab with a selection | `goto:3:1 shift-down shift-tab cmd-s` | the same | **BB-BUG-015** |
| F Cmd+Down, Cmd+Up | `goto:3:5 cmd-down x cmd-up y cmd-s` | `yclass C ... }\nx` | PASS: document ends |

Not covered: a test *run* from the Dock (the Tests panel's Run needs a click,
BB-TG-007; listing from the Dock environment is BBEOD20's readback).

## After BB-BUG-014 and BB-BUG-015 (branch build, the same eight cases)

| Case | Saved | Verdict |
|---|---|---|
| C2 Cmd+Backspace | `beta + gamma;` | PASS: deleted to the line start |
| E1 Tab with a selection | `        int alpha = beta + gamma;` (line 4 untouched: the selection ended at its start) | PASS: indented, nothing lost |
| E2 Shift+Tab with a selection | `int alpha = beta + gamma;` | PASS: outdented |
| A, B, C1, D, F | as above | PASS: unchanged by the new key arms |

## BB-BUG-013: horizontal scroll, branch build, the headmate's own file

`PipelineConfigContractTests.cs` (MbTx, 79 KB), read-only (`--no-write`),
launchd's PATH; `state` now prints `scroll_x`.

| Step | `state` |
|---|---|
| `goto:205:1` | `line=205 col=1 scroll_x=0` |
| End on the 98-char line 205 | `col=97 scroll_x=193.2`: the text moved left, the caret in sight |
| Home | `col=17 scroll_x=0`: the indented line start shows whole |
| `goto:193:92` (the method name's end) | `scroll_x=151.0` |
| `goto:193:1` | `scroll_x=0` |

Installed at `7ab28aa` (the merged fix), same file, launchd's PATH: the
`state` after End printed `scroll_x=0` and the one after Home `193.2`, one
step late even with 3 s pauses. The Mac had been idle 30 min and the
display was asleep: the caret-in-sight scroll runs in the paint path, and
no frame was painted between a key and its `state` (BB-TG-010's family).
The branch build, read back on the awake display, matched its window
tests; the installed build is the same code.

Before the fix the same keys left `scroll_x` absent: the caret went under
the pane's edge (the headmate's screenshot). A trackpad swipe is proven by
the window test's simulated scroll event; by hand on the installed app it
is the headmate's check after the install.

## Installed at `c145669` (both fixes merged): the eight cases again

All eight PASS on the installed app, launchd's PATH; C2, E1 and E2 were the
red ones at `0b6406d`.

## Sweep 2 (installed at `c145669`): selections, Enter, joins, select all

Fresh copies of `Words.cs`, the saved bytes judged.

| Case | Keys | Verdict |
|---|---|---|
| G1 select all, type | `cmd-a x cmd-s` | **BB-BUG-016**: Cmd+A does nothing; `x` went in at the cursor |
| G2 Shift+Right x3, type | `goto:3:5 shift-right shift-right shift-right y cmd-s` | PASS: `    y alpha = beta + gamma;` |
| G3 Shift+End, type | `goto:3:5 shift-end z cmd-s` | PASS |
| G4 Shift+Down, type | `goto:3:1 shift-down w cmd-s` | PASS: the line replaced by `w` |
| G5 Enter keeps the indent | `goto:3:30 enter q cmd-s` | PASS: `    q` on the new line |
| G6 Backspace at a line start | `goto:4:1 backspace cmd-s` | PASS: joins the line above |
| G7 undo after a replace | `cmd-a x cmd-z cmd-s` | PASS (Cmd+A being a no-op, the undo only removed `x`) |
| G8 Cmd+/ | `goto:3:5 cmd-/ cmd-s` | **BB-BUG-017**: no line comment toggle; the file is unchanged |
| G9 Cmd+Shift+Down, type | `goto:3:5 cmd-shift-down x cmd-s` | PASS: `    x` ends the file |
## Sweep 3 (installed at `c145669`): code intelligence keys

Fresh copies of the hand-check solution (writable: the rename saves), Roslyn
waited for, `state` judged; `Square` is declared on line 15, `Half` on 21.

| Case | Keys | Verdict |
|---|---|---|
| H1 F12 on `Square(3)` | `goto:7:29 f12` | PASS: the tab lands on line 15 |
| H2 Shift+F12 on `Square` | `goto:15:23 shift-f12` | PASS: `refs=Some(3)` |
| H3 Find in Files, regex | `cmd-shift-f /Square/ enter` | PASS: results listed |
| H4 Go to Symbol `Half` | `cmd-f12 Half enter` | PASS: line 21 |
| H5 Cmd+. on a line | `goto:9:9 cmd-.` | PASS: actions listed |
| H6 F2 rename `Half` to `Halve` | `f2 Halve enter enter cmd-s` | **BB-BUG-018**: both files read `HalfHalve` (the box appended); the rename itself reached the closed file. **BB-BUG-019**: `problems=11` after the save and still 8 s later; a fresh load shows `0` and `dotnet build` is clean |

Two script errors on the way, not bugs: the rename box is two-step (Enter
previews, Enter applies), and line numbers of the exception copy were used.

