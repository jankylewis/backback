# BB-UIX-019 — fix

id: BB-UIX-019
family: UIX
status: FIXED
source: branch `fix/away-run`; `Breakpoints::for_session`, `for_start`

## The fix that holds

The only way to read breakpoints for a session is `for_session`: it
drops files no longer on disk and returns them for the status line.

## The wrong fix

Filtering in `begin` only: the store keeps the dead path and the next
reader (a breakpoints list) shows it again.
