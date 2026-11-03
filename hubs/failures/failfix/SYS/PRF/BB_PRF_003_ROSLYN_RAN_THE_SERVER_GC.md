# BB-PRF-003 — fix

id: BB-PRF-003
family: PRF
status: FIXED
source: branch `bb-bug-021-move-copy-lines` (`roslyn::start`)

## The fix that holds

Start Roslyn with `DOTNET_gcServer=0` (the workstation GC) unless the
user set the variable. A/B on MbTx, 14 warm runs each: completion p50
5.7 -> 6.4 ms, references 1.03 -> 1.02 s, definition 0.79 -> 0.68 ms,
load 7.1 -> 6.9 s; Roslyn memory after 12 rounds 2108 -> 655 MB.

## The wrong fix

`DOTNET_GCHeapHardLimit`: a hard cap turns a big solution's peak into an
out-of-memory crash of the server instead of a slower collection.

## Proof

Branch build, 12 rounds, no override: 491 -> 655 MB, flat from round 6;
`docs/plan/bench/bb_GENERAL/RERUNS/bb_NIGHT_2026-11-04.md`.
