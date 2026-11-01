# Goals board: every BBEOD and its state

The public copy of the fleet's goals board (TaskHub). Updated with each merge.
Last update: 2026-11-01. Bugs and failures: `hubs/TRIAGE.md`. Scope law: pure C# IDE, minimal features (`AGENTS.md`).

## Status

| Unit | Goal | State | Evidence |
|---|---|---|---|
| BBEOD01 | M0 Engine proof: `bb` CLI, conventions gate, Roslyn loads a real solution | Done | [#1](https://github.com/jankylewis/backback/pull/1) |
| BBEOD02 | M1 Editor: GPUI window, dark default, Explorer, tabs, edit/save, highlighting | Done | [#2](https://github.com/jankylewis/backback/pull/2), [#3](https://github.com/jankylewis/backback/pull/3) |
| BBEOD03 | M2 Code intelligence via roslyn-language-server | Done | [#4](https://github.com/jankylewis/backback/pull/4) |
| BBEOD04 | M3 Test automation, ApexTestkit-compatible; Test Explorer | Done | [#4](https://github.com/jankylewis/backback/pull/4), [#5](https://github.com/jankylewis/backback/pull/5) |
| BBEOD05 | M4 Debugging via netcoredbg, Rider-style Set Next Statement | Done | [#6](https://github.com/jankylewis/backback/pull/6) |
| BBEOD06 | M5 Harness | Reduced | MCP panel [#7](https://github.com/jankylewis/backback/pull/7) removed again in [#9](https://github.com/jankylewis/backback/pull/9) |
| BBEOD07 | M6 macOS release | Partial | [#8](https://github.com/jankylewis/backback/pull/8): `.app` bundle, mascot, CI and release workflows |
| BBEOD08 | Settings window, Menlo 13 on every surface, tool window side | Todo | BB-BUG-002 |
| BBEOD09 | Explorer: git-ignored files and folders shown in pink | Done | BBEOD09 PR; readback `docs/plan/readback/FIXES/S01/BBEOD09/` |
| BBEOD10 | Standard macOS keys: Cmd+M minimize, Cmd+H hide, Home/End | Done | BBEOD10 PR; readback `docs/plan/readback/FIXES/S01/BBEOD10/` |
| BBEOD11 | Integrated terminal: minimal, native, fast, never laggy | Todo | Requested |
| BBEOD12 | **Critical:** cursor shifts the text; draw it as an overlay | Done | BBEOD12 PR; readback `docs/plan/readback/FIXES/S02/BBEOD12/` |
| BBEOD13 | Core C# loop: Open Solution (Cmd+O), Build + error list, Go to File, Go to Symbol, Back/Forward | Partial | Open, recent, Go to File done (BB-BUG-006); Build, Go to Symbol, Back/Forward to do |
| BBEOD14 | Zoom the whole app: Cmd+= in, Cmd+- out, Cmd+0 reset | Todo | Requested |

## Gaps behind a Done or Partial state

- BBEOD06: integrated terminal and find in files were never built. Propose
  before building (scope law).
- BBEOD07: the `.app` is not signed or notarised yet (`sign_release.sh` is
  ready, no certificate run); no Homebrew cask, no v1.0 tag or GitHub release.
- CI installs pinned Roslyn and netcoredbg (BB-CIP-001). It skips the 9
  tests that need the private Apex CLI (BB-TG-004); the local gate runs them.
- Test Explorer clicks did not respond in one real-app readback; not yet
  confirmed as a product bug.

## Next, in order

1. BBEOD12 (cursor, done), then CI green on `master` (done).
2. Install one app from `master` to `/Applications` (BB-BLD-001).
3. Verify and merge the two branches above; BBEOD10.
4. BBEOD13, BBEOD08 with BBEOD14, BBEOD11, BBEOD09.

## BBEOD08 and BBEOD09 in short

- BBEOD08: one font and size for every surface, default Menlo 13; Settings
  window (Cmd+,) with font family and size, live apply, saved in
  `~/Library/Application Support/backback/settings.json`, reset to default;
  tool window on the left or right, default left.
- BBEOD09: the Explorer draws every entry that git ignores in a pink theme
  colour; an ignored folder colours its whole subtree; git decides what is
  ignored (all ignore files and excludes); no pink outside a git repo.
