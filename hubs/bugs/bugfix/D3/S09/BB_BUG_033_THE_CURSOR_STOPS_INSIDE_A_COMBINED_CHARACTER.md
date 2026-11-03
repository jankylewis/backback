# BB-BUG-033 — fix

id: BB-BUG-033
family: UIX
status: FIXED
source: `bb_crates/bb_core/bb_text/src/graphemes.rs` (`before`, `after`), `document/movement/mod.rs` (`left`, `right`), `document/edit/mod.rs` (`backspace`, `delete`)

## The change

Left, Right, Backspace and Delete step one extended grapheme cluster
(`unicode-segmentation`, already in `Cargo.lock`, now a direct
dependency), read from a 64-char window around the cursor so the cost
does not grow with the line. `\r\n` is one grapheme, so the CRLF rule
of BB-BUG-010 is the same rule now.

## Proof

Unit test `steps_cross_a_combining_accent_an_emoji_and_crlf_whole`
(a, e + U+0301, an emoji, a five-char family emoji, CRLF: the same stops
both ways); every `bb_text` test (57) and editor window test (42) green.
Readback on the branch build, `emoji.cs`: six Lefts from the line end
stop at col 21 (before `é`; col 22 before the fix, inside it); two Rights
from before `a` step over `a` and `👍` (col 19).
