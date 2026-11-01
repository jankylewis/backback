# BB-BLD-002 — The `.app` opened no window; a first-frame readback said it did

id: BB-BLD-002
family: BLD
status: FIXED
date: 2026-11-01
signature: .app opens no window
source: commit `2151199` "Fix GUI release bundle and verify served app" (PR #7)
source: `docs/release/scripts/verify_bundle.sh`
links: fix=yes lesson=yes evidence=no toolgap=none

## What it is

The bundle's `CFBundleExecutable` was not the GUI binary, so opening the `.app`
showed no window. A readback that read a "first frame" signal passed anyway:
a false positive, reported as working and then corrected.

## How to confirm

Read `Contents/Info.plist` `CFBundleExecutable`; open the `.app` and see a window.
