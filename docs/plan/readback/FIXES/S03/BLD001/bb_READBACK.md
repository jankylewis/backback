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

Re-installed after PR #21 merged: `install.sh` exit 0, `BBCommit` = `7bc9b96`,
the binary holds the `alacritty_terminal` code, the app runs at nice 0, and
quit leaves no process. The terminal itself is read back in `S02/BBEOD11`. Full-screen captures
are not kept: other apps' windows were on screen.
