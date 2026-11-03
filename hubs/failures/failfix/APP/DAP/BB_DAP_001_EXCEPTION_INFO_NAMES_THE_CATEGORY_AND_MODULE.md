# BB-DAP-001 — fix

id: BB-DAP-001
family: DAP
status: FIXED
source: branch `bb-dap-exception-breaks`

## The fix that holds

`ExceptionInfo::headline` reads the type from `details.fullTypeName` and the
message from `details.message` when the adapter gives them; without
`details` it drops the id's category (the text before `/`) and uses the
description. VS Code's exception widget shows the same two things.

## The wrong fix

Parsing the message out of the description (`: '...'` at its end): a message
holding a quote or a colon would break it, and the type would still carry
`CLR/`.

## Proof

`the_headline_is_the_full_type_then_the_message_as_vs_code_shows_it` on the
recorded reply; `the_all_filter_pauses_on_a_caught_throw_and_names_the_exception`
on the real netcoredbg: `System.InvalidOperationException: total was 14`.
