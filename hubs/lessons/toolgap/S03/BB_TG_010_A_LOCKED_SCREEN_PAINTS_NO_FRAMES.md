# BB-TG-010 — a locked screen paints no frames; script mouse input misses

id: BB-TG-010
for: none
status: OBSERVED
date: 2026-11-03
source: `ioreg -n Root -d1`: `CGSSessionScreenIsLocked` true at 01:06 +07

While the Mac's screen is locked, the app draws no frames, so elements have
no bounds and the readback script's `hover:`, `click:`, `cmdhover:` and
`cmdclick:` steps hit nothing (the cursor stays at 1:1, no question is
sent). Key steps (`BB_KEYS` keys) still reach the focused view.

Workaround: check `CGSSessionScreenIsLocked` before a mouse readback; prove
mouse behaviour with GPUI window tests (no screen needed) and rerun the
real-app check once the screen is unlocked.
