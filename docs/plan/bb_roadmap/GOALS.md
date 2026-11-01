# Goals board: every BBEOD and its state

The public copy of the fleet's goals board (TaskHub). Updated with each merge.
Last update: 2026-11-02. Bugs and failures: `hubs/TRIAGE.md`. Scope law: pure C# IDE, minimal features (`AGENTS.md`).

## Status

| Unit | Goal | State | Evidence |
|---|---|---|---|
| BBEOD01 | M0 Engine proof: `bb` CLI, conventions gate, Roslyn loads a real solution | Done | [#1](https://github.com/jankylewis/backback/pull/1) |
| BBEOD02 | M1 Editor: GPUI window, dark default, Explorer, tabs, edit/save, highlighting | Done | [#2](https://github.com/jankylewis/backback/pull/2), [#3](https://github.com/jankylewis/backback/pull/3) |
| BBEOD03 | M2 Code intelligence via roslyn-language-server | Done | [#4](https://github.com/jankylewis/backback/pull/4) |
| BBEOD04 | M3 Test automation, ApexTestkit-compatible; Test Explorer | Done | [#4](https://github.com/jankylewis/backback/pull/4), [#5](https://github.com/jankylewis/backback/pull/5) |
| BBEOD05 | M4 Debugging via netcoredbg, Rider-style Set Next Statement | Done | [#6](https://github.com/jankylewis/backback/pull/6) |
| BBEOD06 | M5 Harness | Reduced | MCP panel [#7](https://github.com/jankylewis/backback/pull/7) removed again in [#9](https://github.com/jankylewis/backback/pull/9) |
| BBEOD07 | M6 macOS release | Partial | [#8](https://github.com/jankylewis/backback/pull/8): `.app` bundle, mascot, CI and release workflows; install to `/Applications` [#22](https://github.com/jankylewis/backback/pull/22), readback [#24](https://github.com/jankylewis/backback/pull/24) |
| BBEOD08 | Settings window, Menlo 13 on every surface, tool window side | Done | [#19](https://github.com/jankylewis/backback/pull/19), REAP [#17](https://github.com/jankylewis/backback/pull/17); readback `docs/plan/readback/FIXES/S02/BBEOD14/` |
| BBEOD09 | Explorer: git-ignored files and folders shown in pink | Done | [#15](https://github.com/jankylewis/backback/pull/15); readback `docs/plan/readback/FIXES/S01/BBEOD09/` |
| BBEOD10 | Standard macOS keys: Cmd+M minimize, Cmd+H hide, Home/End | Done | [#16](https://github.com/jankylewis/backback/pull/16); readback `docs/plan/readback/FIXES/S01/BBEOD10/` |
| BBEOD11 | Integrated terminal: minimal, native, fast, never laggy | Done | [#21](https://github.com/jankylewis/backback/pull/21); readback `docs/plan/readback/FIXES/S02/BBEOD11/` |
| BBEOD12 | **Critical:** cursor shifts the text; draw it as an overlay | Done | [#12](https://github.com/jankylewis/backback/pull/12); readback `docs/plan/readback/FIXES/S02/BBEOD12/` |
| BBEOD13 | Core C# loop: Open Solution (Cmd+O), Build + error list, Go to File, Go to Symbol, Back/Forward | Done | [#18](https://github.com/jankylewis/backback/pull/18) (Open, Go to File), [#20](https://github.com/jankylewis/backback/pull/20) (Build, Go to Symbol, Back/Forward) |
| BBEOD14 | Zoom the whole app: Cmd+= in, Cmd+- out, Cmd+0 reset | Done | [#19](https://github.com/jankylewis/backback/pull/19) (same change) |

## Gaps behind a Done or Partial state

- BBEOD06: the terminal is built (BBEOD11); find in files is not. Proposal
  P1 in `PROPOSALS.md` waits for the headmate (scope law).
- BBEOD07: install from `master` works (BB-BLD-001 FIXED). Not signed or
  notarised, no v1.0 tag or GitHub release: proposal P2, needs a Developer ID.
- CI installs pinned Roslyn and netcoredbg (BB-CIP-001). It skips the 9
  tests that need the private Apex CLI (BB-TG-004); the local gate runs them.
- BB-BUG-004 (Test Explorer clicks): window test passes; the agent seat cannot
  post real clicks (BB-TG-007). A hand check is proposal P3.

## Next, in order

1. Headmate rulings on `PROPOSALS.md` (P1 find in files, P2 release, P3 hand check).
2. Build what is approved, with red-first tests and a real-app readback.

## BBEOD08 and BBEOD09 in short

- BBEOD08: one font and size for every surface, default Menlo 13; Settings
  window (Cmd+,) with font family and size, live apply, saved in
  `~/Library/Application Support/backback/settings`, reset to default;
  tool window on the left or right, default left.
- BBEOD09: the Explorer draws every entry that git ignores in a pink theme
  colour; an ignored folder colours its whole subtree; git decides what is
  ignored (all ignore files and excludes); no pink outside a git repo.
