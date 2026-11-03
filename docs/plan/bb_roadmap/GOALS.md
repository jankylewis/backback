# Goals board: every BBEOD and its state

The public copy of the fleet's goals board (TaskHub). Updated with each merge.
Last update: 2026-11-03. Bugs and failures: `hubs/TRIAGE.md`. Scope law: pure C# IDE, minimal features (`AGENTS.md`).

## Status

| Unit | Goal | State | Evidence |
|---|---|---|---|
| BBEOD01 | M0 Engine proof: `bb` CLI, conventions gate, Roslyn loads a real solution | Done | [#1](https://github.com/jankylewis/backback/pull/1) |
| BBEOD02 | M1 Editor: GPUI window, dark default, Explorer, tabs, edit/save, highlighting | Done | [#2](https://github.com/jankylewis/backback/pull/2), [#3](https://github.com/jankylewis/backback/pull/3) |
| BBEOD03 | M2 Code intelligence via roslyn-language-server | Done | [#4](https://github.com/jankylewis/backback/pull/4) |
| BBEOD04 | M3 Test automation, ApexTestkit-compatible; Test Explorer | Done | [#4](https://github.com/jankylewis/backback/pull/4), [#5](https://github.com/jankylewis/backback/pull/5) |
| BBEOD05 | M4 Debugging via netcoredbg, Rider-style Set Next Statement; in the app on Ctrl+Shift+F10 and Debug > Set Next Statement, only on an adapter that advertises it (a local patched netcoredbg; nothing published) | Done | [#6](https://github.com/jankylewis/backback/pull/6), [#64](https://github.com/jankylewis/backback/pull/64); readback `docs/plan/readback/V1/BBEOD05/SETNEXT.md` |
| BBEOD06 | M5 Harness: terminal + Find in Files (MCP panel removed by ruling) | Done | Find in Files [#27](https://github.com/jankylewis/backback/pull/27), terminal [#21](https://github.com/jankylewis/backback/pull/21); MCP [#7](https://github.com/jankylewis/backback/pull/7) removed in [#9](https://github.com/jankylewis/backback/pull/9); readback `docs/plan/readback/FIXES/S03/BBEOD06/` |
| BBEOD07 | M6 macOS release | Partial | [#8](https://github.com/jankylewis/backback/pull/8): `.app` bundle, mascot, CI and release workflows; install to `/Applications` [#22](https://github.com/jankylewis/backback/pull/22), readback [#24](https://github.com/jankylewis/backback/pull/24) |
| BBEOD08 | Settings window, Menlo 13 on every surface, tool window side | Done | [#19](https://github.com/jankylewis/backback/pull/19), REAP [#17](https://github.com/jankylewis/backback/pull/17); readback `docs/plan/readback/FIXES/S02/BBEOD14/` |
| BBEOD09 | Explorer: git-ignored files and folders shown in pink | Done | [#15](https://github.com/jankylewis/backback/pull/15); readback `docs/plan/readback/FIXES/S01/BBEOD09/` |
| BBEOD10 | Standard macOS keys: Cmd+M minimize, Cmd+H hide, Home/End | Done | [#16](https://github.com/jankylewis/backback/pull/16); readback `docs/plan/readback/FIXES/S01/BBEOD10/` |
| BBEOD11 | Integrated terminal: minimal, native, fast, never laggy | Done | [#21](https://github.com/jankylewis/backback/pull/21); readback `docs/plan/readback/FIXES/S02/BBEOD11/` |
| BBEOD12 | **Critical:** cursor shifts the text; draw it as an overlay | Done | [#12](https://github.com/jankylewis/backback/pull/12); readback `docs/plan/readback/FIXES/S02/BBEOD12/` |
| BBEOD13 | Core C# loop: Open Solution (Cmd+O), Build + error list, Go to File, Go to Symbol, Back/Forward | Done | [#18](https://github.com/jankylewis/backback/pull/18) (Open, Go to File), [#20](https://github.com/jankylewis/backback/pull/20) (Build, Go to Symbol, Back/Forward) |
| BBEOD14 | Zoom the whole app: Cmd+= in, Cmd+- out, Cmd+0 reset | Done | [#19](https://github.com/jankylewis/backback/pull/19) (same change) |
| BBEOD15 | Cmd + hover underlines a symbol with a definition; Cmd+click goes there | Done | [#29](https://github.com/jankylewis/backback/pull/29); readback `docs/plan/readback/FIXES/S03/BBEOD15/` |
| BBEOD16 | File type icons: JSON, XML, MSBuild, YAML, Razor, SQL, shell, JS/TS, web, images, config, Docker, text | Done | [#30](https://github.com/jankylewis/backback/pull/30); readback `docs/plan/readback/FIXES/S04/BBEOD16/` |
| BBEOD17 | Debugger in the app: breakpoints, F5, Call Stack, Variables, stepping | Done | Breakpoints [#31](https://github.com/jankylewis/backback/pull/31), session [#33](https://github.com/jankylewis/backback/pull/33); readback `docs/plan/readback/FIXES/S03/BBEOD17/` |
| BBEOD18 | Clickable stack traces in test results: each frame opens its call site | Done | [#34](https://github.com/jankylewis/backback/pull/34); readback `docs/plan/readback/FIXES/S04/BBEOD18/` |
| BBEOD19 | Exception breakpoints: Debug menu and panel ticks for All / User-Unhandled; a stop names the exception | Done | [#46](https://github.com/jankylewis/backback/pull/46); readback `docs/plan/readback/V2/BBEOD19/` |
| BBEOD20 | Test Explorer builds a never-built test project once on expand, then lists it (P4); `--no-write` refuses and says so | Done | [#49](https://github.com/jankylewis/backback/pull/49); readback `docs/plan/readback/V2/BBEOD20/` |
| BBEOD21 | Find in Files and Go to File search tracked dot-folders (P5); `.git`, `.vs`, `.idea` and `.gitignore` still rule | Done | [#50](https://github.com/jankylewis/backback/pull/50); readback `docs/plan/readback/V2/BBEOD21/` |
| BBEOD22 | Go to Implementation (Cmd+Alt+B: one place jumps, several open the places panel) and Go to Type Definition (Cmd+Ctrl+B); plain Ctrl+B still moves left | Done | [#57](https://github.com/jankylewis/backback/pull/57); readback `docs/plan/readback/V3/BBEOD22/` |
| BBEOD23 | Start page (P6): a Dock or Finder launch shows the recent workspaces still on disk, newest first, and Open...; Up/Down/Enter or a click opens one, the page closes; File > Start Page... brings it back; an argument still opens directly | Done | [#63](https://github.com/jankylewis/backback/pull/63); readback `docs/plan/readback/V3/BBEOD23/` |
| BBEOD24 | Show Call Hierarchy (Shift+Alt+H): the callers of the symbol at the cursor, every call site in the places panel (one caller still lists), none says so | Done | [#62](https://github.com/jankylewis/backback/pull/62); readback `docs/plan/readback/V3/BBEOD22/BBEOD24_CALL_HIERARCHY.md` |

## Gaps behind a Done or Partial state

- BBEOD07: install from `master` works (BB-BLD-001 FIXED). Distributed as
  source; signing and notarising dropped (P2 ruling 2026-11-03, no paid ID).
- CI installs pinned Roslyn and netcoredbg (BB-CIP-001). It skips the 9
  tests that need the private Apex CLI (BB-TG-004); the local gate runs them.
- Mouse-only paths were proven by window tests, not by hand: the seat cannot
  post clicks (BB-TG-007) and the screen was locked overnight (BB-TG-010):
  Test Explorer Run (BB-BUG-004), Cmd+click, a Call Stack row, a stack-trace
  link. A hand check of all four is proposal P3.
- The installed app is read back with launchd's environment, as the Dock
  starts it (BB-TG-012): `docs/plan/readback/FIXES/S03/BLD001/INSTALL_98F0FCD.md`.

## Next, in order

1. Hubs: cause, fix and lesson pages for BB-BUG-008..012.
2. Build approved P4 (build before test discovery) and P5 (tracked dot-folders).
3. P3 hand checks with the headmate; fix what they find.
4. Readback and perf loops on the installed app; fix what they find.

## BBEOD08 and BBEOD09 in short

- BBEOD08: one font and size for every surface, default Menlo 13; Settings
  window (Cmd+,) with font family and size, live apply, saved in
  `~/Library/Application Support/backback/settings`, reset to default;
  tool window on the left or right, default left.
- BBEOD09: the Explorer draws every entry that git ignores in a pink theme
  colour; an ignored folder colours its whole subtree; git decides what is
  ignored (all ignore files and excludes); no pink outside a git repo.
