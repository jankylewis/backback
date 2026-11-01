# BBEOD11 readback: the terminal

Date 2026-11-02, by backback_mm. Branch `bbeod-11-terminal`.

Minimal by design (headmate: speed and minimalism over UI): the user's login
shell in the solution folder under the editor, Ctrl+backtick (also View >
Terminal menu). `alacritty_terminal` parses on its own thread; the panel draws
only the visible rows, in the one Menlo font, with a block cursor. Keys:
text, Enter, Backspace, Tab, Esc, arrows (app-cursor aware), Home/End,
PageUp/Down, Ctrl+letter, Alt+key; Cmd+V pastes (bracketed when asked).
Scrollback 2,000 lines.

## Proof

| Check | Result |
|---|---|
| Real shell under a PTY (`a_typed_command_runs_and_its_output_reaches_the_screen`) | `echo bb_$((40+2))_ok` -> `bb_42_ok` on the parsed screen; resize to 40x6 -> 6 rows |
| Heavy output (`a_hundred_thousand_lines_stream_through_quickly`) | `seq 1 100000` and the prompt back in 636 ms |
| Window to shell (`keys_typed_in_the_window_run_in_the_shell`) | Ctrl+backtick opens it; keys typed in the window run (`echo bb42ok`); toggle hides it |
| Real app | Ctrl+backtick started `login -flp <user> /bin/zsh` under the app; 11 keys sent, none refused; after quit the shell and Roslyn were gone (owned group, BB-PRC-001) |

## Not covered

- A window capture of the panel: the window was behind another window and
  the capture showed a stale first frame (BB-TG-005).
- Colours (text is the theme's one colour), selection and mouse are left out
  on purpose; propose before adding.
