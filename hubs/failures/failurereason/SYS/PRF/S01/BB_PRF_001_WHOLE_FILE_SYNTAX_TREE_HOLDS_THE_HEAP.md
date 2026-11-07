# BB-PRF-001 — A whole-file syntax tree held 97% of the heap on a 20 MB file

id: BB-PRF-001
family: PRF
status: FIXED
date: 2026-10-31
signature: RSS 1,008 MB with the 20 MB file
source: commit `03a5511` (on `master` via PR #4)
links: fix=yes lesson=yes evidence=no toolgap=none

## What it is

Opening a 20 MB file built one tree-sitter tree for the whole file: 862 MB,
about 41 bytes per source byte. Rope and highlight path were small.

## How it shows

RSS about 1,008 MB; open takes 2.7 s; typing p99 about 2.5 ms of UI work.
