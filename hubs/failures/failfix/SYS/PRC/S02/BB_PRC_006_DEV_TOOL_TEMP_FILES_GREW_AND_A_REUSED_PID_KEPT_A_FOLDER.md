# BB-PRC-006 — fix

id: BB-PRC-006
family: PRC
status: FIXED
source: branch `cli-temp-leak`

## The fix that holds

- The scratch folder is an `OwnedTempDir`, `backback-obj-<pid>-<solution>`:
  drop removes it on every return path, and the sweep family
  `backback-obj` removes what a crash left.
- `create_log` keeps only the newest 50 `open-*.log` files.
- A folder's pid owns it only if that process started no later than the
  folder's last write (`ps -o etime=`, 5 s slack); doubt still keeps it.

## The wrong fix

Removing the log folder at the end of each run: the printed log path is
how a slow load is read afterwards.
