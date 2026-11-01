# BBEOD09 readback: git-ignored entries in pink

Date 2026-11-02, by backback_mm. Release `backback` from `bbeod-09-ignored`,
`--no-write`.

## Runs

1. The headmate's folder (a private repo, opened read-only): its three
   `.env.*` files showed pink, the two `.cs` files kept their colours. `git
   check-ignore` agreed. The capture stays local (private file names).
2. A throwaway repo with `.gitignore` = `.env.*`, `bin/`, `*.log`,
   `!keep.log` (`EXPLORER_PINK.png`): `bin` and `run.log` pink; `keep.log`
   (negated), `Program.cs` and `.gitignore` normal. `git check-ignore`:
   `bin`, `run.log` ignored; `keep.log` not.

Only names are shown; no `.env` file was opened or read.

## Rules (tests in `bb_core/explorer/tests/git_ignored.rs`)

Git decides: every `.gitignore` from the repo root down, `.git/info/exclude`
and the global excludes file. An ignored folder makes its subtree pink.
Outside a git work tree nothing is pink. The colour is the theme token
`ignored` (dark `#f5a9c8`, light `#c7509a`), softer than `error`.
