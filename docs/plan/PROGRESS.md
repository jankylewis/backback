# BBEOD02-A progress

- Seat: `backback_2`; branch: `bbeod-02-editor`.
- Own worktree: `backback_wt/bbeod-02-editor` (AgentFarm claim held).
- Brief: Explorer model, GPUI desktop shell, live readback, performance, full gate.
- State: GPUI builds with `runtime_shaders`; desktop window verified on fixture.
- `xcrun --find metal` exits 72; active developer path is CommandLineTools;
  `/Applications/Xcode.app/Contents/Developer` is absent.
- Explorer model: 7 focused tests passed; desktop keyboard test passed.
- First frame median: 152 ms; 10 s idle RSS median: 109808 KiB.
- MbTx expand-all median: 576 ms for 8,758 folders.
- Fixture screenshot: `docs/plan/readback/BBEOD02_FIXTURE.png`.
- MbTx and Apex read-only window screenshots are saved in `docs/plan/readback/`.
- Live synthetic fixture confirmed nested folders, C# icon, dimmed build folders,
  selected outline, mouse expansion, and keyboard Right navigation.
- Full workspace gate passed: fmt, clippy, 62 tests, and deny. Repo audit clean.
- Path tooltip has not appeared in a screenshot; live appearance is unverified.
- Hub X path named in `AGENTS.md` was absent on disk at session start.
