# BB-GAT-011 — fix

id: BB-GAT-011
family: GAT
status: FIXED
source: `lexer.rs` `skip_escape`, used by `skip_string` and `skip_char`

## The fix that holds

One `skip_escape` for every escape: it keeps an escaped line break.

## Proof

`every_literal_and_comment_keeps_its_line_breaks` (strings, chars, raw,
block and line comments, C# verbatim and regular): red before (1 of 3
breaks kept), green after; `every_public_head_in_this_repo_is_on_the_map`
green again.

## The wrong fix

Rewriting the fixture without line continuations: the map would stay
wrong for the next file that has one.
