# BBEOD02-A Explorer and visual readback

- date: 2026-11-01
- tester: backback_2
- provider: cda
- seat: backback_2
- area: backback Explorer model, macOS desktop shell, and visual review
- predecessor: `../bbeod02_gpui_blocker/SUMMARY.md`
- successors: none

## Intent

Complete the first desktop window and Explorer after the pinned GPUI build failed
on a CommandLineTools-only host.

## What we did

1. Enabled `gpui_platform/runtime_shaders` on the pinned GPUI revision; the desktop binary built.
2. Completed a lazy folder and `.slnx` Explorer with sorting, kinds, dimming, and property tests.
3. Built a dark GPUI Explorer window with mouse expansion and keyboard navigation.
4. Captured fixture, MbTx, and Apex window readbacks, and fixed observed tree and navigation defects.
5. Measured five fresh MbTx launches, idle RSS, and five expand-all runs after warm-ups.
6. Ran the full workspace gate and repository audit; both passed.
7. Reopened after backback_mm found placeholder icons, missing indent guides,
   and incorrect hidden and ignored entry handling.
8. Saw a failing `.DS_Store` model test, added ignore rule tests, then fixed
   metadata visibility and inherited/nested `.gitignore` matching.
9. Drew seven small SVG icons and added a guide line for each tree depth.
10. Expanded MbTx through `MbAuto/RgvMbDesktop/Tests/Documents/Pdf`, refreshed
    the MbTx and synthetic fixture screenshots, and compared stated criteria.
11. Rebenchmarked expand-all, optimized the extra ignore-file lookup, then ran
    the full gate and audit again.

## Key findings

- Reopened `cargo test --workspace`: 65 passed, 0 failed.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo deny check` passed.
- `bb audit ..`: clean.
- First frame median: 152 ms; idle RSS median: 109808 KiB; MbTx expand-all median:
  576 ms for 8,758 folders.
- Runtime shader work is included in first-frame time; its isolated cost is
  unmeasured because full Xcode is absent.
- The path tooltip is attached to each GPUI row, but its live appearance was
  not visible in captured hover states.
- The refreshed MbTx screenshot shows the requested expansion depth, SVG folder
  icons, guide lines, dimmed ignored folders, and a selected row outline.
- The refreshed fixture screenshot shows C#, Markdown, project, solution, and
  generic SVG icons; `.git` and `.DS_Store` are absent and ignored content is dim.
- MbTx expand-all now reaches 8,712 folders in a 664 ms median after warm-up;
  this is 15.3% above the original 576 ms result, with host quietness unverified.

## Decisions

- Keep the GPUI pin and document runtime shader compilation in ADR-2.
- Record the tooltip observation limit instead of treating its presence in code
  as proof of live appearance.
- Leave PR creation and merge to backback_mm.
- Use authored SVGs in place of font glyph placeholders; no external icon pack
  was copied.

## Open items carried forward

- UI: verify and, if needed, fix live path-tooltip appearance on hover.
- Performance: compare runtime shader cost with a precompiled build when full
  Xcode is available; this host cannot produce that build.
- Performance: repeat the new expand-all comparison on a certified quiet host.

## Files created today

- `bb_crates/bb_core/explorer/`: Explorer model and tests.
- `bb_crates/shell/desktop/`: macOS desktop binary and navigation.
- `bb_crates/shell/desktop/assets/`: seven authored SVG icon assets.
- `docs/plan/readback/BBEOD02.md`: readback observations and lessons.
- `docs/plan/bench/BBEOD02.md`: performance measurements.
- `docs/plan/readback/BBEOD02_FIXTURE.png`: fixture window evidence.
- `docs/plan/readback/BBEOD02_MBTX.png`: MbTx window evidence.
- `docs/plan/readback/BBEOD02_APEX.png`: Apex window evidence.
- `rawctxhub/ctx_summarizing/backback/11_01_2026/bb_bbeod02_explorer_ready/SUMMARY.md`:
  sanitized handoff record.
