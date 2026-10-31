# Backback verification and Explorer review

date: 2026-11-01
tester: backback_2
provider: cda
seat: backback_2
area: BBEOD01 independent verification; BBEOD02-A Explorer review round 2
predecessor: ../bb_bbeod02_explorer_ready/SUMMARY.md
successors: none

## Intent

Independently verify the frozen BBEOD01 engine revision, then correct the deep Explorer row layout, truncation, resize behavior, and icon presentation.

## What we did

1. Created a detached external worktree at the frozen BBEOD01 revision and ran the full gate.
2. Ran doctor, scan, audit, three-run bench, and open against the harness; checked fixture timestamps and worktree cleanliness; removed the worktree.
3. Ran one malformed or hostile input for each CLI command, including outline.
4. Added a failing row layout test, then fixed guide widths and checked depths 0 through 12.
5. Added name ellipsis, a draggable Explorer edge, a distinct project SVG, and theme tokens for icon kinds.
6. Reopened the synthetic fixture and read-only MbTx solution in the live macOS window; captured normal and widened Explorer states.
7. Ran the BBEOD02 full gate and repository audit.

## Decisions

- Fixed-width guides and icon slots preserve depth when long names compete for horizontal space.
- The name gets the remaining panel width and GPUI ellipsis; the panel drag range is 180 to 800 px.
- Icon colours are semantic theme tokens; ignored entries use the dim text token.

## Key findings

- BBEOD01 gate: fmt, all-feature clippy, workspace tests, and deny exited 0; 83 tests passed, 0 failed. The harness open left no fixture path newer than the pre-open marker. Hostile inputs returned controlled nonzero exits; no bug was observed.
- BBEOD02 gate: fmt, clippy, workspace tests, and deny exited 0; 66 tests passed, 0 failed across 25 suite results. Repository audit was clean.
- Live MbTx capture shows the five Pdf children one depth below Pdf, with aligned guides. The long first child has an ellipsis at 300 px and fits after dragging to 500 px.

## Open items carried forward

- Tooltip appearance remains outside this review's verified screenshots.
- BBEOD02 performance comparison remains indicative because the earlier host was not certified quiet; the previously recorded ignore-matching cost was not changed in this round.

## Files created today

- `bb_crates/shell/desktop/src/row_view.rs` — fixed Explorer row layout and depth test.
- `docs/bb_design/BBEOD02_RESIZED.png` — live widened MbTx Explorer capture.
- `rawctxhub/ctx_summarizing/backback/11_01_2026/bb_bbeod01_verify_bbeod02_review2/SUMMARY.md` — this sanitized handoff.
