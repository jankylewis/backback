# BB-PRF-006 — Find in Files read one file at a time

id: BB-PRF-006
family: PRF
status: FIXED
date: 2026-11-07
signature: a search over MbTx (3,700 files) took 220-255 ms on one core
source: self-measured (perf epic, 2026-11-07), `find_bench` on MbTx
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The walk listed files on one thread (87 ms on MbTx), and each file was
read and matched after the previous one: the other cores sat idle.
