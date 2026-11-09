# BB-BUG-058 — fix

id: BB-BUG-058
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/panels/terminal/view/bb_mouse.rs` (`wheel`), `bb_session.rs` (`scroll`, `input`)

## The change

The grid takes the scroll wheel: whole lines go to alacritty's
`scroll_display` (a trackpad's small steps add up in `scroll_rest`). In a
full-screen app (alternate screen: less, vim) the wheel sends Up/Down keys
instead, as VS Code does. Typed or pasted input scrolls back to the prompt.

## Lesson

A terminal is not done when it keeps history: the history needs a way in.
Check each input a user expects (wheel, trackpad, keys) against VS Code.

## Proof

`the_wheel_scrolls_back_through_the_output_and_a_key_comes_back` (window
test): red before (40 lines up, `300` still on screen), green after.
`in_a_full_screen_app_the_wheel_sends_arrow_keys`: `cat -v` shows `^[[A`;
red when the alternate-screen branch is removed.

The window test was flaky once in the gate: it waited for a `300` row,
and keys typed before the shell started echo as `seq 1 ` / `300` rows.
It now waits for `299`, a row only seq's output makes. Green in the gate.
