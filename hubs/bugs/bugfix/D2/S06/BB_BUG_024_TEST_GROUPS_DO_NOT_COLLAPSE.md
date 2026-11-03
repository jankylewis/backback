# BB-BUG-024 — fix

id: BB-BUG-024
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/panels/tests_panel/view/rows/bb_groups.rs` (`group_row`), `bb_crates/shell/desktop/src/bb_explorer/panels/tests_panel/mod.rs` (`Group`, `TestsState::toggle`)

## The change

Namespace, class and parameterised-method heads are rows of their own: a
click folds or opens them (▸ / ▾), per project, kept in
`TestsState::collapsed`. A folded namespace hides its classes; a folded
class hides its specs. The class's Run keeps its place on the class row.

## Proof

Window test `namespace_class_and_method_heads_fold_and_open` (all three
kinds, both ways). The test was written with the fix, not run red first:
before the fix the heads had no selector and no handler.
