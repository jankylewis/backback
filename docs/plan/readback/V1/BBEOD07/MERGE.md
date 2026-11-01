# BBEOD07-MERGE readback

Date: 2026-11-01. Seat: `backback_2`. Branch: `bbeod-07-release`.
Merged `origin/master` `6b5e68b` into release base `2151199`. No push.

## Conflict resolution

| Conflict | Resolution |
|---|---|
| Four BBEOD02 readback files: `bb_CORE` vs `bb_EXPLORER` | Kept master's `bb_EXPLORER` copies. Both sides had the same SHA-1 for each file. |
| Readback folder fanout after merge | Grouped BBEOD01 and BBEOD02 under `bb_FOUNDATION`. Moved BBEOD02 editor records under `bb_EXPLORER/EDITOR`, leaving one marker. |

Master's MCP panel group cleanup and tests merged without source conflict.
The release scripts, CI, icon, and README release steps remain on this branch.
The first full gate failed its repository self-audit on two folder rules.
The readback folder had six entries. BBEOD02 had two `bb_` markers.
The three-test self-audit passed after the moves. This is the merge fanout
family; the repository self-audit guards the full tree.

## Icon and docs from verify bus 068

- The earlier record incorrectly called the pack mark `</>`; it showed
  two chevrons without a slash. The SVG now draws the slash.
- The fringe path has a curved left edge. The face outline is clipped below
  the hair, so its top-left arc does not cut across the fringe.
- `icon.icns` has 10 PNG sizes. The dark contact sheet labels all 10.
- `AGENTS.md` gate text matches CI, including all-feature Clippy and audit.
- `README.md` names optional `netcoredbg` and `BACKBACK_NETCOREDBG`.

## Verification

- The post-commit full gate passed: format, all-feature Clippy, 301 tests
  in 46 suites, 0 failed, 0 ignored, deny, and `bb audit ..`.
- Earlier attempts hit the 30-second Roslyn probe deadline under load near
  11. A direct probe took 41.7 seconds. Both probes passed in the final gate.
- The tracked absolute-path guard and `git diff --check` passed.
- Rebuilt app bundle: `codesign --verify --deep --strict` and `plutil -lint`
  passed. The bundled icon matches the asset and extracts to 10 PNG sizes.
- `verify_bundle.sh` found the GUI executable and visible window `20495`.
  [MERGE_WINDOW.png](MERGE_WINDOW.png) shows the scratch C# fixture.
  Quit left 0 bundle processes; 0 matching PPID-1 children remained.
