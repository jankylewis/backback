# BB-BUG-065 — fix

id: BB-BUG-065
family: TST
status: FIXED
source: `editor/view/chrome/test_marks.rs` (`short_name`)

## The change

`target` compares the method's own name (the last part of the full name)
with the declared name, so the ▶ and the new right-click menu find their
test.

## Lesson

A fallback can hide a broken main path: the ▶ "worked" (it opened the
panel), so nobody saw it never ran a test. Test the main path, not only
that a click does something.

## Proof

`a_right_click_in_a_test_method_in_the_editor_offers_run_and_debug`: the
gutter's ▶ runs the test; with the old comparison the test fails.
