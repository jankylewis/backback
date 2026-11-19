# BB-UIX-005 — The key map ignored Option and Ctrl

id: BB-UIX-005
family: UIX
status: FIXED
date: 2026-11-03
signature: Option+Backspace deletes a single char
source: `bb_crates/shell/desktop/src/editor/input/bb_keys.rs`; bugs BB-BUG-011, BB-BUG-012
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The editor's key map matched the key name alone (`left`, `backspace`,
`a`), so Option+Left was Left, Option+Backspace was Backspace, and
Ctrl+A/E/B/F/P/N/H/D were nothing (a Ctrl letter has no `key_char`, so no
text went in either). Two whole families of macOS text keys were missing:
the word moves and deletes, and the Emacs-style line keys every macOS text
field has.

## How it shows

Option+Right moves one character; Option+Backspace deletes one; Ctrl+A and
Ctrl+E leave the cursor where it was. Found by a family sweep after
BB-BUG-010: every modifier with every editing key.

## How to confirm it

Option+Right at a word start must land after the word; Ctrl+E mid-line
must land at the line end.
