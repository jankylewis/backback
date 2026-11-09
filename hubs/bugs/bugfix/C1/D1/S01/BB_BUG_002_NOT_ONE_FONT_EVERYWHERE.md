# BB-BUG-002 — fix

id: BB-BUG-002
family: UIX
status: FIXED
source: `bb_crates/shell/ui/src/typography.rs` (`Typography`)
source: `bb_crates/shell/desktop/src/bb_explorer/panels/settings/`
source: `docs/plan/readback/FIXES/S02/BBEOD14/bb_READBACK.md`

## The change

One `Typography` (family, size, zoom, tool window side), default Menlo 13,
saved as a versioned settings file. Only the window root sets the font; every
surface inherits it. The editor row, caret, gutter and click maths read the
same value. Settings window (Cmd+,), zoom (Cmd+= / Cmd+- / Cmd+0).

## Proof

Red first: the `Typography` tests and the source scan
`no_surface_sets_its_own_font_family_or_text_size`. Live capture at 13 pt;
window tests for zoom and the Settings window.
