# BB-UIX-017 — fix

id: BB-UIX-017
family: UIX
status: FIXED
source: branch `fix/nav-back-closed-file`; `editor/bb_tabs/model/history/bb_stack.rs` (`lives`, `hop`)

## The fix that holds

One rule, `lives`: a place is reachable while its tab is open or its
file is on disk. The arrow (`Tabs::can_go`) and the press (`Tabs::hop`)
both use it. `here` is optional; a press drops the gone places on its
way, names them, and moves `here` to the other stack only when it goes.

## The wrong fix

Greying ← when the tab closes, or dropping its places from the history:
VS Code reopens a closed editor on Go Back, and that is what the
headmate approved. Checking only the disk: an open untitled or
deleted-but-open tab would be skipped.
