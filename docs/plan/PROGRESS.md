# BBEOD01-C docs progress

- Seat: `backback_2`; owned tree: `docs/`.
- Branch observed: `bbeod-01-engine`.
- Docs plan, conventions, and research notes are ready for handoff.
- `cargo run -q -p bb -- audit ..` returned `audit : clean (..)`.
- Source evidence: AgentFarm goals board and two BBEOD01-B spikes.

- BBEOD01-E: renamed plan marker to `bb_roadmap/`; independent frozen worktree gate pending.

## BBEOD07-A release work

- Seat: `backback_2`; branch: `bbeod-07-release`; owned worktree: `backback_wt/bbeod-07-release`.
- AgentFarm claim acquired. Brief and four required skills loaded.
- Scope: original mascot assets, macOS CLI app bundle, CI, release workflow, and community docs.
- Current product is an M0 CLI without a GUI. App screenshots are not yet available.
- Foreseen bug families: version drift (derive from Cargo), invalid icon sizes (render and inspect each size), missing bundle file (assert paths), CI drift (run the local gate), and documentation overclaim (state the M0 limit).
- Hub X path named in instructions was absent on this machine; standing repo laws and loaded skills apply.
- Bundle built and ad-hoc signature verified; bundled `bb doctor` and two `bb scan` readbacks passed.
- Full gate passed: fmt, clippy, 83 tests, deny. `bb audit` clean after a README cap correction.
- Bench ran five samples, but load 3.24 missed the quiet threshold. No valid perf verdict.
- Final audit clean. Release artifacts and evidence are ready for branch commit and handoff.
- Open items: no GUI screenshots at M0; quiet benchmark baseline and rerun; Apple Developer ID signing and notarization by the headmate.
- BBEOD07-A polish: removed the hair arc over the goggles and separated the pack strap at the right shoulder; re-exported all icon sizes.
