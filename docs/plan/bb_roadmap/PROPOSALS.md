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

## P3 — Test Explorer hand check (BB-BUG-004)

- What: one click by hand on Tests -> a project -> Run in the installed app.
- Why: the agent seat cannot post clicks (BB-TG-007); the window test passes.
  A hand click closes the bug or gives a real repro.
