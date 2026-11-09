# BB-BUG-025 — fix

id: BB-BUG-025
family: UIX
status: FIXED
source: `bb_crates/edge/dotnet/testkit/src/model/source.rs` (`find_source`), `bb_crates/shell/desktop/src/bb_explorer/panels/tests_panel/view/rows/spec.rs` (`open_spec`)

## The change

A click on a spec's row shows its result and opens its source.
`bb_testkit::find_source` reads the project folder's `.cs` files (not
`bin`, `obj`, dot folders; at most 12 deep and 5000 files) off the UI
thread: the class's declaration (the test's namespace wins), then the
first line after it that names the method as a whole word before `(` or
`<`, else the declaration. No file declaring the class is said in the
status bar.

## Proof

Seven unit tests on `find_source` (method line, parameterised case,
inherited method, namespace choice, build output skipped, longer names and
calls on other objects, nested and generic classes, no file). Window test
`a_click_on_a_spec_opens_its_method`. Readback (branch build, scratch copy of the hand-check solution, `bug024_readback.py`): `testopen:Half_of_5`
opened `MathOpsTests.cs` line 14, the method's line.
