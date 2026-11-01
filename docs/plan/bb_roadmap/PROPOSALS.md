# Proposals awaiting the headmate

Scope law: extras are proposed here, never built unasked. Each one says what,
why, cost and how it would be proven. Approve, change or drop.

## P1 — Find in Files (Cmd+Shift+F), the rest of BBEOD06

- What: one search box over the open root; plain text or regex, case
  toggle; results grouped by file with line previews; Enter jumps (Back
  returns, as for F12). Skips the same folders as Go to File (`.git`,
  `bin`, `obj`, `node_modules`) and git-ignored files.
- How: the `ignore` + `grep-searcher` crates (ripgrep's engine, MIT/Unlicense)
  on a background thread; results stream into the existing `PlacesPanel`
  (the Build error list), capped at 2,000 hits; a new query cancels the old.
- Cost: about 1 BBEOD. One dependency family (ripgrep crates, `cargo deny`
  clean expected).
- Proof: red-first tests on a temp tree (match, regex, skip, cap, cancel);
  window test: type a query, Enter opens the file at the line; real-app
  readback on a large solution with the time to first result.

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
