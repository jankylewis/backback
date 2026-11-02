# BB-CIP-003 — lesson

id: BB-CIP-003
family: CIP
status: FIXED
source: branch `bb-cip-003-yanked`

## Who believed what

A green local `cargo deny` meant CI's would be green too.

## What changed the belief

Yank state lives in the crates.io index; the local cache was older than a
yank made the same day, so only CI saw it.

## Rule left behind

When CI's deny fails on `yanked` and the local one passes, ask crates.io
(`/api/v1/crates/<name>/versions`) and update that one package.
