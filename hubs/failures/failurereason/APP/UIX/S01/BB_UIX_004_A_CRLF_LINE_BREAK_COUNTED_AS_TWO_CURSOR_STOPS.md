# BB-UIX-004 — A CRLF line break counted as two cursor stops

id: BB-UIX-004
family: UIX
status: FIXED
date: 2026-11-03
signature: a lone \r in a saved CRLF file
source: `bb_crates/bb_core/bb_text/src/document/movement/mod.rs`, `edit/mod.rs`; bug BB-BUG-010
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The text model moved and deleted by `char`, and `\r` and `\n` are two
chars. Right at a line end stopped between them; Backspace at a line start
removed only the `\n`; Delete at a line end only the `\r`. Typing at that
stop put text between the two bytes, so the saved file held a lone `\r`
and a line break no tool reads as one.

## How it shows

Only on CRLF files (common in C# repositories from Windows): the cursor
shows col 9 on an 8-char line; after a save, `git diff` shows the whole
line changed and a `^M`.

## How to confirm it

`python3 -c "print(repr(open(f,'rb').read()))"` on the saved file: a `\r`
not followed by `\n`.
