# Proposals awaiting the headmate

Scope law: extras are proposed here, never built unasked. Each one says what,
why, cost and how it would be proven. Approve, change or drop.

## P1 — Find in Files (Cmd+Shift+F): DONE

Built 2026-11-03 as the rest of BBEOD06 (headmate: "continue ... until all
BBEODs completed"). `/re/` regex, smart case, `.gitignore` aware, 2,000-hit
cap, newer search wins. One crate, `bb-search` (`ignore` + `regex`; the
`grep-searcher` crate was not needed). Readback
`docs/plan/readback/FIXES/S03/BBEOD06/`, bench `docs/plan/bench/BBEOD06/FIND.md`.

## P2 — Signed and notarised release, rest of BBEOD07

- What: `sign_release.sh` run with a Developer ID; notarise; `v1.0.0` tag;
  GitHub release with the `.app` zip; later a Homebrew cask.
- Needs from the headmate: an Apple Developer ID certificate on this Mac
  (or a CI secret) and a go for the first public tag.
- Proof: `spctl -a -vv` accepts the app; a fresh download opens with no
  Gatekeeper warning.

## P3 — Four hand checks in the installed app

- What, by hand, in the installed app on a C# solution:
  1. Tests -> a project -> Run (BB-BUG-004);
  2. hold Cmd over a method call: it underlines; Cmd+click goes there;
  3. F9 on a line, F5, then click the second Call Stack row: its call site;
  4. run a failing test, click a blue stack-trace line: its file and line.
- Why: the agent seat cannot post clicks (BB-TG-007) and the screen was
  locked overnight (BB-TG-010); window tests pass for all four.
