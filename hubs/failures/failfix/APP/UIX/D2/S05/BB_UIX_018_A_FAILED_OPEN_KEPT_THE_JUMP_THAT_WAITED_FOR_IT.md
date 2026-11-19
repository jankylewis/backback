# BB-UIX-018 — fix

id: BB-UIX-018
family: UIX
status: FIXED
source: branch `fix/nav-back-closed-file`; `editor::open_failed`

## The fix that holds

Both open error arms call `open_failed`: drop the jumps waiting for that
path and say `Cannot open <name>: <error>`.

## The wrong fix

Checking `is_file()` before storing the jump: a file can go between the
check and the load, and an unreadable file passes the check.
