# BB-GAT-001 — a property test asked more than tree-sitter promises

id: BB-GAT-001
family: GAT
status: FIXED
date: 2026-11-03
signature: incremental_parse_equals_a_fresh_parse ... FAILED (minimal input of 5 edits)
source: CI run 41532817663 (PR #39, docs only), `bb_core/syntax/src/bb_highlight.rs:207`
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The property test asserted that after every random edit the incrementally
reparsed tree equals a fresh parse, exactly, even when the text is broken.
Tree-sitter promises that only for valid text: in broken code its error
recovery may keep a different but valid tree for an edit. proptest found
such a case on CI: a `{` typed at the end of already-broken code.

## How it shows

Red on a docs-only PR. The edit points backback hands tree-sitter were
checked against an independent count (bytes, `\n` rows, columns): all
equal, including a CRLF split. The divergent tree re-converged on the next
edit, and every valid state matched a fresh parse.
