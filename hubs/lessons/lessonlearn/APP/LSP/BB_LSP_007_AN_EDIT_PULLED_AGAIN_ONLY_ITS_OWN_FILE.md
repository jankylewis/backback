# BB-LSP-007 — lesson

id: BB-LSP-007
family: LSP
status: FIXED
source: branch `bb-bug-019-diag-refresh`

## Who believed what

A file's diagnostics change only when that file changes, so pulling the
edited file again is enough (the server thread's `changed` set).

## What changed the belief

A rename: one edit, two files, and the other file's errors stayed. A C#
diagnostic is a fact about the solution that is shown per file.

## Rule left behind

Ask again for a list whenever what it is computed from changes: lists of a
whole-solution analysis are pulled again for every open file, on every
change and on every server refresh.
