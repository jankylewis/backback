# Goals board: every BBEOD and its state

The public copy of the fleet's goals board (TaskHub). Updated with each merge.
Last update: 2026-11-01. Scope law: pure C# IDE, minimal features (`AGENTS.md`).

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
| BBEOD08 | Settings window, one font everywhere (Menlo 13), tool window side | Todo | Parked |
| BBEOD09 | Explorer: git-ignored files and folders shown in pink | Todo | Parked |

## Gaps behind a Done or Partial state

- BBEOD06: integrated terminal and find in files were never built. Propose
  before building (scope law).
- BBEOD07: the `.app` is not signed or notarised yet (`sign_release.sh` is
  ready, no certificate run); no Homebrew cask, no v1.0 tag or GitHub release.
- CI on `master` is red since #8: the runner lacks roslyn-language-server and
  netcoredbg, so three Roslyn worker tests fail. Local gate: 270 passed, 0 failed.
- Test Explorer clicks did not respond in one real-app readback; not yet
  confirmed as a product bug.

## Committed, not merged

| Branch | Content | Local gate |
|---|---|---|
| `bbeod-03-diagver` | Roslyn readiness: count loaded projects once, wait on readiness not a fixed time | 289 / 0 |
| `bbeod-08-reap` | No child processes left after quit (Roslyn, test host, debugger) | 304 / 0 |

Each needs an independent verify before its PR.

## Next, in order

1. CI green on `master`.
2. Verify and merge the two branches above.
3. Confirm or fix the Test Explorer click readback.
4. BBEOD08, then BBEOD09.

## BBEOD08 and BBEOD09 in short

- BBEOD08: one font and size for every surface, default Menlo 13; Settings
  window (Cmd+,) with font family and size, live apply, saved in
  `~/Library/Application Support/backback/settings.json`, reset to default;
  tool window on the left or right, default left.
- BBEOD09: the Explorer draws every entry that git ignores in a pink theme
  colour; an ignored folder colours its whole subtree; git decides what is
  ignored (all ignore files and excludes); no pink outside a git repo.
