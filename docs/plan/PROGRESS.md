# BBEOD02 progress

- Hygiene-ctx (`backback_2`, 2026-11-01): moved five tracked ctx summaries
  unchanged to the external rawctxhub; removed the repo copies and added an
  ignore rule and external-hub instruction. Public-path review found two
  absolute home paths; a tracked-text guard failed on both before removal.
  Revised gate: fmt, Clippy, 126 tests, deny, and `bb audit` passed.
## BBEOD05-A debugger engine in progress

- Seat: `backback_2`; branch: `bbeod-05-debug`; base: `978f75f`.
- Official netcoredbg `3.2.0-1092` macOS arm64 archive passed its published SHA-256 check.
- Next: typed DAP session, real debugger probe, gate, readback, and benchmark evidence.
- DAP attach readback: GreeterSpec stopped, locals were visible, step resumed, and both tests finished.
- Probe CLI stopped at a temporary console fixture and printed stack and locals.
- Measured attach-to-first-stop 594 ms and step-to-stop 19 ms in one real run.
- Final gate: format, Clippy, 87 tests, and cargo-deny all passed.
- Performance: five-run release benchmark completed at load1 5.02; quiet comparison remains open.
- Remaining: handoff summary, commit, and READY to `backback_mm`.

## BBEOD05-A acceptance benchmark

- Functionally accepted; performance comparison is in progress.
- A: master `978f75f`; B: `bbeod-05-debug` `e01277d`.
- Plan: release builds, five interleaved A/B pairs, three measured runs per item.
- Target items: scan-harness, scan-solution, and audit-repo.
- The redundant test-only Clippy allow was removed; `#![cfg(test)]` kept the integration helpers in test scope.
- A/B second window: median paired B deltas +2.7% harness, −1.5% solution, +3.8% audit.
- A/A control and sign-changing pairs support `noise, no regression`.

- Seat: `backback_2`; owned tree: `docs/`.
- Branch observed: `bbeod-01-engine`.
- Docs plan, conventions, and research notes are ready for handoff.
- `cargo run -q -p bb -- audit ..` returned `audit : clean (..)`.
- Source evidence: AgentFarm goals board and two BBEOD01-B spikes.

- BBEOD02-FIX (backback_1, 2026-11-01): fixes for verification bugs B1-B3.
  B1 safe save, B2 rope edit log + background open/parse (key p99 244 ms
  -> 2.6 ms), B3 virtual Explorer (frame 439 -> 197 ms, RSS 471 -> 112 MiB),
  status bar, natural sort, Quit. Evidence: `bench/BBEOD02_FIX.md`,
  `readback/BBEOD02_FIX/`. Open: FS watcher, 1 GB RSS on 20 MB file.

- BBEOD02-B follow-up opened after acceptance: add light theme that follows
  macOS appearance, live View toggle, and screenshots of both modes.
- One rapid-typing performance round remains: compare one change against the
  accepted 500-key release baseline on the same 5,000-line fixture. Keep only
  a faster, green change; record the result in `docs/plan/bench/EDITOR.md`.
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
- Editor evidence: `docs/plan/bench/EDITOR.md` and
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
