# BBEOD02-A progress

- Seat: `backback_2`; branch: `bbeod-02-editor`.
- Own worktree: `backback_wt/bbeod-02-editor` (AgentFarm claim held).
- Brief: Explorer model, GPUI desktop shell, live readback, performance, full gate.
- State: GPUI builds with `runtime_shaders`; desktop window verified on fixture.
- `xcrun --find metal` exits 72; active developer path is CommandLineTools;
  `/Applications/Xcode.app/Contents/Developer` is absent.
- Initial Explorer model: 7 focused tests passed; desktop keyboard test passed.
- First frame median: 152 ms; 10 s idle RSS median: 109808 KiB.
- MbTx expand-all median: 576 ms for 8,758 folders.
- Fixture screenshot: `docs/plan/readback/BBEOD02_FIXTURE.png`.
- MbTx and Apex read-only window screenshots are saved in `docs/plan/readback/`.
- Live synthetic fixture confirmed nested folders, C# icon, dimmed build folders,
  selected outline, mouse expansion, and keyboard Right navigation.
- Initial workspace gate passed: fmt, clippy, 62 tests, and deny.
- Path tooltip has not appeared in a screenshot; live appearance is unverified.
- Reopened after backback_mm review: vector icons, depth guides, hidden metadata,
  git-ignore dimming, and a deep MbTx readback are implemented and captured.
- Refreshed screenshots: `docs/plan/readback/BBEOD02_MBTX.png` (Pdf depth) and
  `BBEOD02_FIXTURE.png` (all authored icon kinds, hidden and dimmed entries).
- Expanded MbTx benchmark median is 664 ms for 8,712 folders; prior median
  was 576 ms for 8,758 folders. The +15.3% indicative cost is recorded.
- Reopened full workspace gate passed: fmt, clippy, 65 tests, and deny; repo audit clean.
- Hub X path named in `AGENTS.md` was absent on disk at session start.
- Review round 2: fixed depth-7 guide shrink with a failing-first row layout
  test covering depths 0–12; added ellipsis, draggable 180–800 px panel edge,
  distinct project SVG, and semantic per-kind icon colours.
- Live MbTx recheck at `MbAuto/RgvMbDesktop/Tests/Documents/Pdf`: child rows
  indent one 16 px cell; 300 px truncates with an ellipsis, and 500 px shows
  the full `_HvFvFromTableFvStandalone` name. Captures are `BBEOD02_MBTX.png`
  and `docs/bb_design/BBEOD02_RESIZED.png`.
