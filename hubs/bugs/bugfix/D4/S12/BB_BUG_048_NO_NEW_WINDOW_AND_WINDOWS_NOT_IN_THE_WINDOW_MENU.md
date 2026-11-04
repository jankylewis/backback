# BB-BUG-048 — fix

id: BB-BUG-048
family: UIX
status: FIXED
source: `app/bb_launch/bb_window.rs` (`title`), `app/bb_launch/mod.rs` (`build_window`), `app/bb_launch/open/mod.rs` (`NewWindow`, `bindings`, `file_menu`)

## The change

Every workspace window is titled with its folder's name
(`window.set_window_title`), so macOS lists it under Window and in Cmd+`
order. File > New Window (Cmd+Shift+N) shows the start page, backback's
empty window (an open one comes to the front).

## Lesson

On macOS the Window menu lists only windows that have a title; gpui names
the menu `Window` as NSApp's windowsMenu, so a title is all it takes.

## Proof

`down_and_enter_open_the_second_entry_and_close_the_page` now asks the
opened window's title (`Second`); without the fix it reads `None` (mutation
checked). `new_window_is_first_in_the_file_menu_and_on_cmd_shift_n`.
