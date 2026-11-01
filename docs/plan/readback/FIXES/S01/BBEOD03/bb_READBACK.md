# BBEOD03 DIAGVER + LOADCOUNT readback

Date 2026-11-01, by backback_mm, after rebasing both commits on `master`.

## Setup

- App: release `backback` from `bbeod-03-diagver`, `--no-write`, on a copy
  of the `_Harness` fixture (3 projects), opened at `_NUnitApex/GreeterSpec.cs`.
- `BB_KEYS`: `wait:ready`, `state`, `click:1:1`, `z z z`, wait, `state`,
  `backspace` x3, wait, `state` (`STATE_LOG.txt`, four runs).

## Result

| Run | Ready | Problems: loaded / typed `zzz` / deleted / later |
|---|---|---|
| 1 | 4.2 s | 12 / 18 / 18 (6 s after) / not checked |
| 2 | yes | 12 / 18 / 12 (5 s) / 12 (20 s) |
| a | yes | 12 / 12 (typing did not land) / 12 / 12 |
| b | yes | 12 / 18 / 12 (6 s) / 12 (16 s) |

The fixture holds 12 problems on purpose. Every run reached Ready with all
projects loaded. Typing `zzz` adds 6 problems; deleting it brings back 12 in
runs 2 and b. No run ended on a stale list.

## Open

- Run 1 still showed 18 problems 6 s after the deletion (load ~8-10), and
  had no later check: BB-LSP-003 (OPEN), not proven stale or slow.
- Run a: the scripted click did not land (seen also in BBEOD10).
