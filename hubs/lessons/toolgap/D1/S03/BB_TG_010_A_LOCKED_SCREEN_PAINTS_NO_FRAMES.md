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

Addendum 2026-11-03 04:20: with the session locked and the display asleep,
an app started through LaunchServices (`open`) never drew a first frame (0%
CPU, empty stderr, no C# server); a direct exec of the same binary with the
same `env -i` environment drew at once (first frame 86 ms). For locked-screen
readbacks with launchd's environment, exec the binary directly.
