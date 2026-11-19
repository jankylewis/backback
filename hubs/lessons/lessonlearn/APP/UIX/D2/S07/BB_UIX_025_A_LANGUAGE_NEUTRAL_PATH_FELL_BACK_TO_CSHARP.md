# BB-UIX-025 — lesson

id: BB-UIX-025
family: UIX
status: FIXED
source: BBEOD40 family sweep

## Who believed what

Each earlier fix (075, 077, 084, 085, 086, LSP-013) was treated as a point bug; `None => Roslyn` was kept on purpose ("as before the picker").

## What changed the belief

A grep for `C#`, `Roslyn`, `csproj`, `CSHARP` and `unwrap_or(Lang::CSharp)` across the shell found five more decisions with the same default; three were user-visible in a Rust window.

## Rule left behind

A language-neutral decision takes a `Lang` (or `Option<Lang>` with `None` meaning "none", never C#) and maps it through one exhaustive match; "C# as before" holds only where `Langs` says C# is active (`file_mode`).
