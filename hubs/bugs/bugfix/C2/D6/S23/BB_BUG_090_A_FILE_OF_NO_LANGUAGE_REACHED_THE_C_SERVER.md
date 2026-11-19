# BB-BUG-090 — fix

id: BB-BUG-090
family: UIX
status: FIXED
source: branch `fix/csharp-assumption-sweep`

## The change

`server_for(path) -> Option<Server>` through one exhaustive `server_of(Lang)`; `ready_for` false, `status_for` says `NO_SERVER`, `send` drops the request for a file of no language.

## Proof

`a_file_of_no_language_reaches_no_server_in_a_window_of_all_three` and `f12_in_a_file_of_no_language_in_a_rust_window_never_names_csharp` red at 88177da (Cargo.toml open/change/ask in the C# fake; `C#` in the status), green after; M1 (`None` -> Roslyn again) red.
