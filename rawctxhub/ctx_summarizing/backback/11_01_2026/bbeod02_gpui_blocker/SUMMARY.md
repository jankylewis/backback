# BBEOD02-A GPUI build blocker

- date: 2026-11-01
- tester: backback_2
- provider: cda
- seat: backback_2
- area: backback Explorer model and macOS desktop shell
- predecessor: none
- successors: `../bb_bbeod02_explorer_ready/SUMMARY.md`

## Intent

Build the first macOS desktop window and Explorer tree in an isolated worktree.

## What we did

1. Read repository law and the BBEOD02-A brief; loaded four assigned skill sets.
2. Claimed `backback_wt/bbeod-02-editor` and created branch `bbeod-02-editor` at `0c287a8`.
3. Added a lazy folder and `.slnx` Explorer model with unit and property tests.
4. Observed a failing test for macOS `/var` versus `/private/var` path identity and fixed it.
5. Pinned GPUI and its platform crate to Zed revision `ba63a022c9a22ca66d68ef2a8f0bb9b5b0a9984d`.
6. Ran `cargo check -p backback`; the `gpui_apple` build failed during Metal shader compilation.

## Key findings

- `cargo test -p bb-explorer`: 4 passed, 0 failed.
- `cargo clippy -p bb-explorer --all-targets -- -D warnings`: passed.
- `xcrun --find metal`: exit 72; `metal` is unavailable.
- Active developer path: `/Library/Developer/CommandLineTools`.
- Full Xcode at `/Applications/Xcode.app/Contents/Developer`: absent.
- The specified Hub X path was absent on disk during this session.
- The first absolute AgentFarm claim refused with exit 2; a repo-relative claim succeeded. The refusal was reported to backback_mm.

## Decisions

- Stop the GUI task at the brief's GPUI build condition and request a manager ruling.
- Do not report READY, full gate, screenshots, or performance data without evidence.
- Leave the worktree uncommitted so the manager can review the blocked state.

## Open items carried forward

- Environment: provide a full Xcode toolchain with `metal`, then compile and validate the pinned GPUI build.
- Product: complete keyboard and mouse behavior, screenshot readback, performance runs, full gate, and commit.
- Review: inspect the draft desktop code after GPUI compilation; it has not passed type checking.

## Files created today

- `bb_crates/bb_core/explorer/`: lazy Explorer model and tests.
- `bb_crates/shell/desktop/`: draft GPUI desktop binary.
- `rawctxhub/ctx_summarizing/backback/11_01_2026/bbeod02_gpui_blocker/SUMMARY.md`: sanitized handoff record.
