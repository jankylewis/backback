# BB-BUG-074 — fix

id: BB-BUG-074
family: UIX
status: FIXED
source: branch `fix/five-debug-window`; `bb_core/syntax/src/test_marks.rs` (`TestMethod::end_line`), `editor/view/chrome/test_marks.rs` (`holding`)

## The change

A test method spans its first attribute through its closing brace (tree-sitter); the menu opens only on a line in that span. The menu is the Tests panel row's own (same look, same Run / Debug, occluded).

## Lesson

A "nearest above" lookup needs an end, or it claims everything after it.

## Proof

`a_right_click_outside_a_test_method_opens_no_test_menu` (window, right mouse; red before: the class brace opened `Counts(1)`); `a_test_spans_its_attribute_through_its_closing_brace`.
