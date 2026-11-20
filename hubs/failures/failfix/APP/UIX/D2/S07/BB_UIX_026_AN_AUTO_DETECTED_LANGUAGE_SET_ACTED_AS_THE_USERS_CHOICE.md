# BB-UIX-026 — fix

id: BB-UIX-026
family: UIX
status: FIXED
source: branch `bbeod-41-python-opt-in`

## The fix that holds

Ticks come from a user's pick or from `default_ticks`, never stored; only a pick is written, and the record says so (`picked`, store v2). Everything a window shows for a language asks the window's active set: the tab mark follows the tab's `FileMode`, the Build menu and Find in Files take `Langs::active`, the Explorer row takes `python`.

## The wrong fix

Dropping Python from the saved v1 lines, or deleting the user's store: the store cannot tell picks from auto-saves, and the app must never edit what it cannot read as its own.
