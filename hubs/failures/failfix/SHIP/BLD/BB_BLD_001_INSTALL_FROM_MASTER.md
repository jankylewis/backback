# BB-BLD-001 — fix

id: BB-BLD-001
family: BLD
status: FIXED
source: `docs/release/scripts/install.sh`; `BBCommit` key from `bb_bundle.sh`

## The fix that holds

One install path: `install.sh` refuses a non-`master` or dirty checkout and a
running installed app, bundles, and replaces `/Applications/backback.app`.
The installed Info.plist carries `BBCommit`, so a readback names its build:
`/usr/libexec/PlistBuddy -c 'Print :BBCommit' /Applications/backback.app/Contents/Info.plist`.

## The wrong fix

Opening whichever worktree bundle is at hand; it never updates.

## Proof

Real install from `master` `60cfb64`: exit 0, `BBCommit` = `60cfb64`, launched
from `/Applications`, Dock mascot with a running dot, no process left on quit.
Readback: `docs/plan/readback/FIXES/S03/BLD001/bb_READBACK.md`.
