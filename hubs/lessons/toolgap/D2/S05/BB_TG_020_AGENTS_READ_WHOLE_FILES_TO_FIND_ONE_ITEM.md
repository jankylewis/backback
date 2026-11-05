# BB-TG-020 — Agents read whole files to find one item

id: BB-TG-020
status: FIXED
for: none
date: 2026-11-05

## Gap

An agent had no map of the code. To find where a type or function lives,
or what a crate is for, it read whole files: the Rust sources are about
1.6 MB (~400k tokens), and one task often read thousands of lines to use
ten. A hand-written map would go stale with the next PR.

## Workaround

`bb map` (crate `bb-kmap`) reads the source on every call, so it cannot go
stale: `bb map` gives one line per crate (~2 KB), `bb map <path>` each
file's purpose and public items with line numbers (`--all` adds private
ones), `bb map --find <name|Owner::name>` the declaration. The whole repo
outline is ~200 KB, 8x less than the source; a crate is a few KB. The test
`every_public_head_in_this_repo_is_on_the_map` checks the map against a
plain line scan of this repo. `AGENTS.md` tells agents to start with it.
