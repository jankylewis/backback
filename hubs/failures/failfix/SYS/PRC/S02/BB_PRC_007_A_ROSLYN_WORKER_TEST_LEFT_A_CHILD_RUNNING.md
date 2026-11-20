# BB-PRC-007 — fix

id: BB-PRC-007
family: PRC
status: FIXED
source: branch `bbeod-42-no-orphan-servers`, `intel/bb_worker/tests/events.rs` `finish`, `lang_tests/tools/live/mod.rs` `end`

## The fix that holds

The child was the test's own Roslyn (or rust-analyzer): the test returned
while its server thread still owned it. Each worker test now ends with
`finish`: drop the link, drain the event channel to closed (the thread
closes it only after it ended its server), then `ps` finds no
`LanguageServer` child. The live window tests end with `end`: close the
windows, drain each bridge, find no server child.

## The wrong fix

A longer nextest `leak-timeout`, or trusting the BB-PRC-009 guard: it
ends a server after the test process exits, later than nextest's 100 ms.

## Proof

nextest 0.9.148 `-j 1`: before, `LEAK [ 4.199s] … trace::gui_solution_open_and_document_sync_leave_no_trace`
(CI run 44701010716: 4 leaky); after, 3 runs of the 5 worker and
rust-analyzer tests and the 3 live window tests: no LEAK.
