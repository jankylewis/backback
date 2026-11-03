# BBEOD21 readback: tracked dot-folders in search

Date 2026-11-03, by backback_mm. Branch `bbeod-21-search-dot-folders`. P5.

## What

Find in Files and Go to File skipped every hidden folder; VS Code and Rider
search `.github`, `.config` and the like. Now one walker (`bb_search::files`)
serves both: hidden entries are walked, `.git`, `.vs` and `.idea` are
skipped with `bin`, `obj`, `node_modules` and `.target`, `.DS_Store` is
never a result, and `.gitignore` still rules. Go to File gains
`.gitignore` on the way (its own walker only knew `bin`, `obj`, `.git`).

## Result

| Check | Evidence |
|---|---|
| Search: `.github/workflows/ci.yml` and `.config/dotnet-tools.json` found; `.git`, `.vs`, `.idea`, `.DS_Store`, build output, ignored and binary files not | `build_vcs_ide_ignored_and_binary_files_are_skipped_but_dot_folders_are_searched` |
| Go to File: `.github/workflows/ci.yml` and `.gitignore` listed; `.git`, `.idea`, `obj`, a git-ignored `Generated/` not | `build_vcs_ide_and_ignored_files_are_never_listed_but_dot_folders_are` |
| Real app (branch build, `--no-write`, `env -i` with launchd's PATH), MbTx: Cmd+Shift+F, `SearchKey` typed key by key, Enter | `327 results for SearchKey in 3615 files, 1126 ms` = `git grep -n SearchKey` 327 lines (326 in 3,588 files before; the line gained is in the tracked `.claudoholic/` folder). The tree stayed as it was: `git status` shows only two untracked folders dated days before the run |
| Same run: Cmd+Shift+O, `dotnettools`, Enter (MbTx has no `.github`; it tracks `.config`, `.vscode`, `.claudoholic`, `.bench_coldstart`, `.idea`) | the tab opened is `.config/dotnet-tools.json`; `.idea` stays unlisted by design |

## Not covered

The Explorer tree is unchanged: it hides `.git` and `.DS_Store` only, as
before.
