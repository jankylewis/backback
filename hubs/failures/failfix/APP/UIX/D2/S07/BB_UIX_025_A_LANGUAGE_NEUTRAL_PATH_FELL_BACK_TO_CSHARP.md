# BB-UIX-025 — fix

id: BB-UIX-025
family: UIX
status: FIXED
source: branch `fix/csharp-assumption-sweep`

## The fix that holds

One exhaustive per-language rule per decision, so a new `Lang` fails to compile instead of falling to C#: `server_of(Lang)` (a file of no language has no server), `Lang::of_project` (every language's project file), `Langs::file_mode` for every tab the app makes.

## The wrong fix

A `_ => Server::Roslyn` arm, or a `.cs` check at each caller: the next caller copies the C# default again.
