# BB-BLD-004 — fix

id: BB-BLD-004
family: BLD
status: FIXED
source: bb_crates/shell/ui/bb_dock/src/lib.rs, docs/release/bb_assets/icon_tools/bb_make.sh

## The fix that holds

`bb_make.sh` renders the SVG, clears the tile and pixelates the figure on a
64 x 64 grid: `icon.icns` (Finder, Launchpad) and a 256 px `dock.png`
(nhetrinh NT-PRF-001: a 1024 px runtime icon costs memory). At launch,
`bb_dock::set_icon` hands `dock.png` to `setApplicationIconImage`: the only
`unsafe` in backback, in its own crate (Peaker 2026-11-16).

## Wrong fixes

1. A transparent `.icns` alone: the Dock still draws its tile.
2. ImageMagick's own SVG reader: it dropped the strokes, the mouth and the
   hair fringe. `qlmanage` (WebKit) renders the SVG as designed.

## Proof

`bb-dock` tests (the image reads, bad bytes and off-main-thread refused);
the Dock readback of the bundle in PR #110.
