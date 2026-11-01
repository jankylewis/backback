# Backback progress

## BBEOD06-FIX (backback_1, 2026-11-01)

- `bbeod-06-gui` from `9d7ea08`. B3-B8 + F1 fixed test-first; lessons in `readback/BBEOD06/FIX/bb_FIX.md`.

## BBEOD06-B ready for review (backback_2, 2026-11-01)

- Branch `bbeod-06-gui`, claimed worktree `backback_wt/bbeod-06-gui`.
- Engine merge `123d9e2`; fixture path, plan text, and stale fixture consumers resolved.
- Full gate: fmt, Clippy, 136 tests, deny, and `bb audit ..` green.
- Window-ID screenshots show stopped and Greet result. Quiet median-of-five bench remains unverified at load1 4.97.
- Hub X path from `AGENTS.md` is absent on disk; advisory rules apply.

- Hygiene-ctx (`backback_2`, 2026-11-01): moved five tracked ctx summaries
  unchanged to the external rawctxhub; removed the repo copies and added an
  ignore rule and external-hub instruction. Public-path review found two
  absolute home paths; a tracked-text guard failed on both before removal.
  Revised gate: fmt, Clippy, 126 tests, deny, and `bb audit` passed.

- BBEOD02-FIX (backback_1, 2026-11-01): fixes for verification bugs B1-B3.
  B1 safe save, B2 rope edit log + background open/parse (key p99 244 ms
  -> 2.6 ms), B3 virtual Explorer (frame 439 -> 197 ms, RSS 471 -> 112 MiB),
  status bar, natural sort, Quit. Evidence: `bench/editor/BBEOD02_FIX.md`,
  `readback/BBEOD02_FIX/`. Open: FS watcher, 1 GB RSS on 20 MB file.

- BBEOD02-B follow-up opened after acceptance: add light theme that follows
  macOS appearance, live View toggle, and screenshots of both modes.
- One rapid-typing performance round remains: compare one change against the
  accepted 500-key release baseline on the same 5,000-line fixture. Keep only
  a faster, green change; record the result in `docs/plan/bench/editor/EDITOR.md`.
- State first: branch `bbeod-02-editor`, last accepted SHA `7c35450`, no push.
- Theme live readback passed: separate View menu, light toggle, return to the
  current macOS dark appearance through Follow, no restart; both PNGs saved.
- One rapid-key scheduling candidate (`window.refresh`) was measured A/B and
  reverted: p99 41.255 vs 41.222 ms with unequal 11.892/13.332 s key loops.
  Two-frame latency remains open; final full gate passed: fmt, Clippy, 109
  workspace tests, and deny.

- BBEOD02-B editor implemented on `bbeod-02-editor`: rope document, guarded
  tabs and window close, atomic save, visible-line C# highlighting, and input
  latency capture from GPUI's input-to-present histogram.
- Full post-editor gate passed: fmt, clippy, 107 tests, deny. Focused Unicode
  incremental-reparse and short-line vertical cursor regressions passed.
- Live fixture, MbTx, and Apex readbacks passed for tab open and rendering;
  temporary fixture save and dirty window close prompt were verified.
- Review trace: 500 paced keys produced 500 presents, p99 22.036 ms on the
  5,000-line fixture. The active display was 47.95 Hz; each GPUI frame
  reported a 20.833 ms refresh interval. This demonstrates a display floor
  above the 16 ms budget on this configuration; quiet-host check remains open.
- Three live editor screenshots were saved and inspected under
  `docs/plan/readback/BBEOD02/EDITOR/`.
- Editor evidence: `docs/plan/bench/editor/EDITOR.md` and
  `docs/plan/readback/BBEOD02/EDITOR/bb_EDITOR.md`.

- BBEOD02-B start: merged `origin/master` into `bbeod-02-editor`; resolved four
  fixture project rename conflicts in favor of the `_*.csproj` backbone names.
- Pre-editor full gate passed: fmt, clippy, 95 tests, deny. Merge `f99746f`.
- Editor risk families: path identity (typed paths and tab dedup); encoding loss
  (BOM/CRLF round-trip test); undo grouping (word-boundary test); stale syntax
  (edit/reparse span test); cursor bounds (checked rope indices); partial save
  (same-directory temporary file plus rename and failure tests).

- Seat: `backback_2`; branch: `bbeod-02-editor`.
- Own worktree: `backback_wt/bbeod-02-editor` (AgentFarm claim held).
- Brief: Explorer model, GPUI desktop shell, live readback, performance, full gate.
- State: GPUI builds with `runtime_shaders`; desktop window verified on fixture.
- `xcrun --find metal` exits 72; active developer path is CommandLineTools;
  `/Applications/Xcode.app/Contents/Developer` is absent.
- Initial Explorer model: 7 focused tests passed; desktop keyboard test passed.
- First frame median: 152 ms; 10 s idle RSS median: 109808 KiB.
- MbTx expand-all median: 576 ms for 8,758 folders.
- Fixture screenshot: `docs/plan/readback/BBEOD02/BBEOD02_FIXTURE.png`.
- MbTx and Apex read-only window screenshots are saved in `docs/plan/readback/`.
- Live synthetic fixture confirmed nested folders, C# icon, dimmed build folders,
  selected outline, mouse expansion, and keyboard Right navigation.
- Initial workspace gate passed: fmt, clippy, 62 tests, and deny.
- Path tooltip has not appeared in a screenshot; live appearance is unverified.
- Reopened after backback_mm review: vector icons, depth guides, hidden metadata,
  git-ignore dimming, and a deep MbTx readback are implemented and captured.
- Refreshed screenshots: `docs/plan/readback/BBEOD02/BBEOD02_MBTX.png` (Pdf depth) and
  `BBEOD02/BBEOD02_FIXTURE.png` (all authored icon kinds, hidden and dimmed entries).
- Expanded MbTx benchmark median is 664 ms for 8,712 folders; prior median
  was 576 ms for 8,758 folders. The +15.3% indicative cost is recorded.
- Reopened full workspace gate passed: fmt, clippy, 65 tests, and deny; repo audit clean.
- Hub X path named in `AGENTS.md` was absent on disk at session start.
- Review round 2: fixed depth-7 guide shrink with a failing-first row layout
  test covering depths 0–12; added ellipsis, draggable 180–800 px panel edge,
  distinct project SVG, and semantic per-kind icon colours.
- Live MbTx recheck at `MbAuto/RgvMbDesktop/Tests/Documents/Pdf`: child rows
  indent one 16 px cell; 300 px truncates with an ellipsis, and 500 px shows
  the full `_HvFvFromTableFvStandalone` name. Captures are `BBEOD02/BBEOD02_MBTX.png`
  and `docs/bb_design/BBEOD02_RESIZED.png`.

- BBEOD01-E: renamed plan marker to `bb_roadmap/`; independent frozen worktree gate pending.
## BBEOD06-A engine (backback_2, 2026-11-01)

- `bbeod-06-mcp` merged into BBEOD06-B. Discovery, stdio client, CLI, fixture, tests, and guards are in the merge.
- Engine gate: fmt, Clippy, 90 tests, deny; `bb audit ..` clean. Fixture cold tools 1,309 ms; warm call p50 135 µs.
- Readback: MbTx 0, NmcTx 1, SI 1 MCP projects; no real MCP launch. Quiet benchmark remains open.
