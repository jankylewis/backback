# BB-BUG-083 — fix

id: BB-BUG-083
family: UIX
status: FIXED
source: branch `fix/away-run`; `jump_to` (`tests_panel/actions/navigation.rs`)

## The change

A path that does not resolve says `the file no longer exists` when it
maps under the opened root, and `not on this machine` otherwise
(`/nowhere/C.cs` keeps its words).

## Proof

`a_test_rows_go_to_reopens_its_closed_file_and_a_deleted_one_says_why`
now asserts the words: red at 6f40103 (`... B.cs: not on this machine`),
green after; `each_stack_frame_opens_its_own_call_site` kills the
always-"no longer exists" mutation.
