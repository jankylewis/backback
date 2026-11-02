# BBEOD17 readback, part 1: breakpoints

Date 2026-11-03, by backback_mm. Branch `bbeod-17-breakpoints`, release
build, scratch copy of the `_Harness` fixture (`Core/Greeter.cs`),
`--no-write`. Input: `BB_KEYS` keys (the screen was locked, BB-TG-010; key
steps still reach the window). Reference: VS Code and Rider (F9; click the
gutter; breakpoints move with their code).

## Result (`bb_debug_state`, 0-based lines)

| Step | Breakpoints | Expected |
|---|---|---|
| F9 on lines 7 and 9 | [6, 8] | both set |
| Enter at the start of line 2 | [7, 9] | both move down with their code |
| Cmd+Z | [6, 8] | back |
| Backspace at the start of line 6 (joins it into line 5) | [5, 7] | both move up |
| F9 on line 6 | [7] | removed |

The status bar says "Breakpoint set/removed at line N" each time.

## Not covered

A real click on a line number (screen locked): the window test
`a_click_on_a_line_number_toggles_it_and_the_cursor_stays` clicks the drawn
number and checks the cursor did not move. Part 2 (the session: F5, Call
Stack, Variables, stepping) is next.
