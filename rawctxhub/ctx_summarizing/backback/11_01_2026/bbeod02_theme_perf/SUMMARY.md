# BBEOD02 theme and rapid typing follow-up

- Date: 2026-11-01
- Provider: cda
- Seat: backback_2
- Area: backback macOS editor
- Predecessor: `rawctxhub/ctx_summarizing/backback/11_01_2026/bbeod02_editor_ready/SUMMARY.md`

## Intent

Finish the accepted editor follow-up: a live light theme that follows macOS
appearance, one rapid-typing performance round, screenshots, and the full gate.

## Observed work

1. Wired the existing light and dark palette through Explorer and editor views.
2. Added a native View menu toggle and a Follow macOS Appearance action.
3. Found a macOS menu-placement defect on the live surface, wrote a failing
   desktop test, then placed View after the application menu.
4. Opened the C# fixture in a live window, toggled to light, and used Follow
   to return to the current macOS dark appearance without restart.
5. Saved and inspected light and dark window screenshots under
   `docs/plan/readback/BBEOD02/EDITOR/THEME/`.
6. Compared release rapid typing on the same 5,000-line fixture before and
   after one refresh scheduling candidate. p99 was 41.255 versus 41.222 ms;
   the candidate was reverted because the difference was inconclusive and
   its draw p99 was higher.
7. Passed fmt, Clippy, 109 workspace tests, and cargo deny.

## Decision and open item

Keep theme changes and the menu regression test. Rapid typing still reaches
about two refreshes at the observed 20.833 ms interval; frame traces show
500 draws but only 216-229 presentations. The trace does not isolate display
link timing from drawable queueing. The matemaster owns independent review
and the PR decision.
