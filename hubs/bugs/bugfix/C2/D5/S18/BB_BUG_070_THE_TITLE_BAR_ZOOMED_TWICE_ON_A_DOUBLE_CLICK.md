# BB-BUG-070 — fix

id: BB-BUG-070
family: UIX
status: FIXED
source: branch `fix/five-debug-window`; `editor/view/chrome/edges/bb_title.rs` (`TitleDrag`)

## The change

A press only arms a drag; the system drag starts on the first move while the left button is held (a move with the button up disarms: the release fell outside the bar). A double click never arms; its release zooms once. As Zed's `platform_title_bar`.

## Lesson

A platform call that takes over the mouse (a window drag) must not start on a press that may be half of a double click; start it on the move.

## Proof

`a_double_click_zooms_once_and_only_a_moved_press_drags`; mutations (double click arms; move ignores the button) both red. `launch::` menu tests green (Cmd+M, Window > Zoom). The green button and a live double click in the installed app: not verified by this seat.
