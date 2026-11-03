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

Before the fix the same keys left `scroll_x` absent: the caret went under
the pane's edge (the headmate's screenshot). A trackpad swipe is proven by
the window test's simulated scroll event; by hand on the installed app it
is the headmate's check after the install.
