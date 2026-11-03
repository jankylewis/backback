# BB-LSP-007 — An edit pulled diagnostics again only for its own file

id: BB-LSP-007
family: LSP
status: FIXED
date: 2026-11-03
signature: problems=11 after a rename; no squiggle in the edited file
source: `bb_crates/shell/desktop/src/intel/bb_worker/bb_loop/mod.rs` (`handle`, `next`)
observed: sweep 3 case H6 (BB-BUG-019) on the installed app at `c145669`; the red test on master
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The server thread kept one set of documents to pull again: those a
`didChange` touched. A C# diagnostic depends on the whole solution, so an
edit in one file adds or removes errors in another open file, whose list
then stays as it was. A rename hits it every time: `flush` sends the other
file's `didOpen` (with its first pull) before the edited file's
`didChange`, so that first list is computed against the old name, and only
the edited file is pulled again.

The server can say when lists go stale (`workspace/diagnostic/refresh`),
but only to a client that declares the refresh support. bb declared none.
LSP 3.17 names the capability `workspace.diagnostics`; lsp-types 0.95
serializes it as `workspace.diagnostic`, a key the protocol does not have.

## How it shows

Problems counts errors the code no longer has, none in the edited file,
after an edit whose effect crosses files (a rename, a signature change).

## How to confirm it

Open two files, change a member the other one uses, and wait for that
file's `Diagnostics` event: before the fix none comes.

Related: BB-LSP-003 (a deleted error counted 6 s after the edit) may be the
server-side half of this family; the refresh path is its guard, not yet
read back.
