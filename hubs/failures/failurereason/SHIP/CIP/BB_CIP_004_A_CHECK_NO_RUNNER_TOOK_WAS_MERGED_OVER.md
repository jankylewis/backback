# BB-CIP-004 — A check no runner took was merged over

id: BB-CIP-004
family: CIP
status: FIXED
date: 2026-11-06
signature: The job was not acquired by Runner of type hosted even after multiple attempts
source: PR #88, run 42079599220 (cancelled after 15 min, no steps); run 42079606160 green
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

One of the PR's two `gate` runs got no macOS arm64 runner (GitHub capacity)
and ended `fail` after 15 minutes without running a step. The merge job's
guard, `gh pr checks | grep -qv pass`, did not stop it, and PR #88 merged
with one check red. No code went unchecked: the other run and the local
gate (536 passed) ran the same commit, and the rerun of the red run passed.

## How it shows

`gh pr checks` lists `gate fail 15m2s`; the job has `conclusion: cancelled`,
`runner_name` empty and no steps. Why the text guard let it through was not
reproduced; a guard on free text is the defect either way.
