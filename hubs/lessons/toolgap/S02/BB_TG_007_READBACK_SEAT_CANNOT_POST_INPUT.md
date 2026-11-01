# BB-TG-007 — the readback seat cannot post mouse input

id: BB-TG-007
for: BB-BUG-004
status: OBSERVED
date: 2026-11-02
source: `hubs/bugs/bugfacing/S01/BB_BUG_004_TEST_EXPLORER_CLICKS_NO_RESPONSE.md`

From the agent's shell `AXIsProcessTrusted()` is `false`. A `CGEvent` mouse
click posted to the HID tap is dropped: the cursor location before and after
is the same, and a click on the Explorer's `Tests` header changed nothing.
So a real-app readback driven by posted input proves nothing about clicks.

Workaround: prove click paths with GPUI window tests (`click(vcx, id)`), and
in the real app with `BB_KEYS` steps; a hand click by the headmate is the
only real-click proof until the seat is granted Accessibility.
