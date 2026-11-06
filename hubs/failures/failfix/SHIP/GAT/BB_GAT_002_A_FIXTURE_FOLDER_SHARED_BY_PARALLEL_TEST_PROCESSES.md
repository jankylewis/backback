# BB-GAT-002 — fix

id: BB-GAT-002
family: GAT
status: FIXED
source: `bb_crates/edge/dap/tests/support/fixture.rs` (`build`), `bb_core/process/src/temp.rs` (`FAMILIES`)

## The fix that holds

The fixture builds in `bb_process::OwnedTempDir::create_with("bb-dap",
tag)`: `bb-dap-<pid>-<tag>` under the OS temp folder, one per process.
`Built` holds the folder; the next create (and `bb sweep`, `bb-dap` is in
`FAMILIES`) removes the folders of processes that are gone. Wrong fix:
running the debugger tests one at a time on CI to hide the shared folder.

## Proof

Same repro after the fix: `bb-dap` + `bb-process`, 40 of 40 passed with 4
workers; `cargo test -p bb-dap` (the local gate's way) also green.
