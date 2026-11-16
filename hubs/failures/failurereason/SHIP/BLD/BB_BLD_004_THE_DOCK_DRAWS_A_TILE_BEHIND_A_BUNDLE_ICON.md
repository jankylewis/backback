# BB-BLD-004 — The Dock draws a tile behind a bundle icon

id: BB-BLD-004
family: BLD
status: FIXED
date: 2026-11-16
signature: backback sits on a dark rounded tile in the Dock; the headmate's other apps show only their figure
source: headmate report 2026-11-16 (Dock screenshot)
links: fix=yes lesson=yes evidence=yes toolgap=none

## Mechanism

Two causes. The mascot SVG drew its own dark tile (`<rect rx="220">`), and
this macOS draws a bundle icon (`CFBundleIconFile`) on a tile when the image
is not a full tile itself, even with a transparent `.icns` (nhetrinh
NT-BLD-001). An image set at launch with `NSApp.setApplicationIconImage` is
drawn as it is; ARM, fleetmic and nhetrinh all do that.

## Confirm

`strings <binary> | grep setApplicationIconImage` on an app with no tile.
