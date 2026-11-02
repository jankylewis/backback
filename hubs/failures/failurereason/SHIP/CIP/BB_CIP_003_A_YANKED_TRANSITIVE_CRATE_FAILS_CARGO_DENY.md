# BB-CIP-003 — a yanked transitive crate fails `cargo deny` on CI

id: BB-CIP-003
family: CIP
status: FIXED
date: 2026-11-03
signature: error[yanked]: detected yanked crate
source: CI run 41510600740 (PR #33), step "Dependency audit"
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`async-executor` 1.14.0 (via gpui_linux / smol and zbus) was yanked on
2026-11-02 when 1.14.1 shipped. `Cargo.lock` still named 1.14.0, so CI's
`cargo deny check` failed on advisories with `error[yanked]` for every PR
from then on. No backback code changed.

## How it shows

The local gate said `advisories ok`: the yank state comes from the crates.io
index, and the local index cache was older than the yank. Only CI, with a
fresh index, saw it.
