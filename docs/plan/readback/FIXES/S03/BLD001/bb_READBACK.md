# BB-BLD-001 readback: install from master to /Applications

Date 2026-11-02, by backback_mm. Main checkout on `master` at `60cfb64`
(PR #22 merged), clean; no `/Applications/backback.app` before.

## Real install

| Check | Result |
|---|---|
| `bash docs/release/scripts/install.sh` | exit 0; release build 2 m 46 s; `installed /Applications/backback.app (60cfb64)` |
| Build stamp | `PlistBuddy -c 'Print :BBCommit'` -> `60cfb64` = `master` |
| Bundle | `MacOS/backback`, `Resources/bb`, `Resources/backback.icns`; signature verified by the script |
| Launch | `open /Applications/backback.app`: process path `/Applications/backback.app/Contents/MacOS/backback`; 2 windows (Explorer, Open panel: Finder launch asks for a folder) |
| Menu bar | `backback` with File, View, Build, C#, Window |
| Dock | the mascot (`ICON.png`, rendered from the installed `.icns`) with a running dot (`DOCK_MASCOT.png`) |
| Quit | SIGTERM -> no backback or Roslyn process left |

## Not covered

The terminal (PR #21) was not on `master` yet, so this build has no
Terminal menu; re-run `install.sh` after it merges. Full-screen captures
are not kept: other apps' windows were on screen.
