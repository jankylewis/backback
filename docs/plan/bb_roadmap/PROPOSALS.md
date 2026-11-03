# Proposals awaiting the headmate

Scope law: extras are proposed here, never built unasked. Each one says what,
why, cost and how it would be proven. Approve, change or drop.

## P1 — Find in Files (Cmd+Shift+F): DONE

Built 2026-11-03 as the rest of BBEOD06 (headmate: "continue ... until all
BBEODs completed"). `/re/` regex, smart case, `.gitignore` aware, 2,000-hit
cap, newer search wins. One crate, `bb-search` (`ignore` + `regex`; the
`grep-searcher` crate was not needed). Readback
`docs/plan/readback/FIXES/S03/BBEOD06/`, bench `docs/plan/bench/BBEOD06/FIND.md`.

## P2 — Signed and notarised release: DROPPED (2026-11-03)

Headmate ruling: no paid Apple Developer ID. backback ships as source on the
public GitHub repo; users build and install with `install.sh` (README). The
section below is kept as the record of the dropped offer.

### Dropped offer

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

## P4 — Build a never-built test project before listing its tests: DONE (2026-11-03, BBEOD20)

- Today: on a fresh clone, expanding a test project in the Test Explorer
  says "not built: ...dll (build the project first)" (readback 2026-11-03,
  `_Harness` copy). VS Code and Rider build before discovery.
- Offer: when the project has no build output, build it once on expand,
  then list; never in `--no-write` mode (it writes `bin/` and `obj/`).

## P5 — Search tracked dot-folders: DONE (2026-11-03, BBEOD21)

- Today: Find in Files and Go to File skip every hidden folder; MbTx has a
  match in a tracked `.claudoholic/` file that the search does not list.
  VS Code and Rider search folders such as `.github` and `.config`.
- Offer: search tracked dot-folders; still skip `.git`, `.vs`, `.idea`.

## P6 — Start page: DONE (2026-11-03, BBEOD23)

- A launch with no argument from the Dock or Finder shows the recent
  workspaces and Open... instead of reopening the newest one; File > Start Page....

## P7 — Next performance and quality steps (night R&D 2026-11-04)

Measured in `docs/plan/bench/bb_GENERAL/RERUNS/bb_NIGHT_2026-11-04.md`;
none built. Approve, change or drop each.

- P7a Explorer refresh off the UI thread. Today one `read_dir` per loaded
  folder every 2 s on the UI thread (BB-BUG-037). Offer: read in the
  background, merge on the UI thread, or use FSEvents. Proof: a 2 000-folder
  expanded tree keeps key work under 1 ms p99. Cost: half a day.
- P7b Shaped-line cache. Every frame shapes the widest visible line (scroll
  width) and, after a cursor move, the cursor line again (caret). Offer:
  cache shaped lines by text across frames. Proof: key work on a 9 000-char line before and after.
- P7c Linear run building. `styled_line` scans every highlight span for
  each char (chars x spans, bounded by the 10 000-char cap). Offer: a span
  cursor. Proof: highlight time on a dense 10 000-char line.
- P7d Readback timing without App Nap. Wall time per scripted key grew
  80 -> 490 ms while the app idled (BB-TG-018). Offer: the BB_KEYS runner
  holds a user-initiated activity while it runs. Proof: the 5-round tab
  loop at a flat time per key.
- P7e Two key rulings: Cmd+[ / Cmd+] (Back/Forward or VS Code's
  indent/outdent) and Cmd+Shift+O (Go to File or VS Code's Go to Symbol).
