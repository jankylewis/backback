# BB-BUG-092 — fix

id: BB-BUG-092
family: UIX
status: FIXED
source: branch `fix/csharp-assumption-sweep`

## The change

Both take the window's `Langs::file_mode` (the rule `open_file` and `recolour` use); `tab::CSHARP` is test-only now.

## Proof

`a_rust_rename_opens_the_closed_file_it_edits_with_rust_colours` (`Colours(CSharp)`) and `cmd_n_in_a_rust_window_opens_a_plain_new_file_not_a_csharp_one` red at 88177da, green after; M4 and M5 (C# again) red.
