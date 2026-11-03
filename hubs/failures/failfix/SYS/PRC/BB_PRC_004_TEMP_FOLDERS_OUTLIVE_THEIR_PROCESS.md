# BB-PRC-004 — fix

id: BB-PRC-004
family: PRC
status: FIXED
source: branch `bb-res-001-temp-folders`

## The fix that holds

Ownership as for process groups (BB-PRC-001): one type owns the folder,
drop and quit remove it, and what a crash or a static leaves is swept by
pid on the next create and by `bb sweep` after each gate.

## The wrong fix

`tempfile::TempDir` alone: it removes on drop, but a static and a quit
without a join never drop, so the leak stays for exactly the biggest
folders.
