# BB-PRF-007 — The language walk ran on the UI thread before the first frame

id: BB-PRF-007
family: PRF
status: FIXED
date: 2026-11-20
signature: first_frame_ms grows with the folder on a launch with no saved languages
source: `shell/desktop/src/app/bb_launch/open/langs.rs` `resolve` (called from `build_window`)
source: `bb_core/lang/src/detect.rs` `detect` (walk up to `DETECT_BUDGET`, 400 ms)
observed: BBEOD36 A/B, release bundles, `aa1d223` (before the picker) vs `19dd6d3`
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

A launch with a folder argument and no saved language choice (the first
open of a folder, or any folder whose walk never completes) walks the
folder for C#, Rust and Python markers inside `build_window`, on the UI
thread, between the window opening and its first frame. A C#-only folder
never finds Rust or Python, so the walk reads the whole tree (up to the
400 ms budget or 200,000 entries). Before the picker there was no walk.

## How it shows

Medians of 5 after a warm-up, same machine, same hour, load 2.7-3.5:

| Folder | `aa1d223` first frame | `19dd6d3` first frame |
|---|---:|---:|
| MbTx (real, 6,673 tracked files; C# and Python) | 187 ms | 325 ms (+74 %) |
| Synthetic 50,000 `.cs` files (C# only) | 182 ms | 272 ms (+49 %) |
| Same, launched through `open` (LaunchServices) | 247 ms | 316 ms (+28 %) |

The app's own `first_frame_ms` (window open to first frame): 55 -> 189 ms
and 54 -> 139 ms. A saved choice of C# alone (walk stops at the first C#
file) costs the same as before: 56 vs 56 ms.

## How to confirm

Time `bb_lang::detect` on the folder with a small program (BBEOD36, warm
cache: MbTx 113 ms, 50,000 files 74 ms) and compare it with the growth of
`first_frame_ms`; the two match.
