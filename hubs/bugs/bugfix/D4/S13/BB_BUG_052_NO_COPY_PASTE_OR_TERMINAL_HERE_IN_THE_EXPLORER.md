# BB-BUG-052 — fix

id: BB-BUG-052
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/files/` (`Item`, `Menus`, `view.rs`: `offered`; `bb_ops.rs`: `copy_into`; `jobs/bb_paste.rs`), `panels/terminal/mod.rs` (`open_in`, `cd_line`)

## The change

Four new rows. Open Terminal Here starts the shell in the row's folder (a
file's own folder); if the shell already runs, it gets Ctrl+U and
`cd -- '<folder>'`, quoted for any name (the panel keeps one shell). Copy
remembers the entry; Paste copies it, off the UI thread, into the row's
folder under its own name or the first free `name copy`, `name copy 2`
(a folder whole, links kept as links, never into itself), then selects
it. Copy Path puts the full path on the clipboard. The root row offers
New, Open Terminal Here, Paste and Copy Path. `--no-write` refuses Paste
as it refuses every change; Copy, Copy Path and the terminal still work.

## Lesson

A `cd` typed into a running shell must clear the half-typed line first
(Ctrl+U) and single-quote the path with `'\''` for a quote: a folder
named `it's $here` otherwise breaks the line or expands `$here`.

## Proof

Unit: free copy names, whole-folder copy with a link, into-itself
refused; a real shell `cd`s into `it's $here` (mutation-checked: fails
without the quote escape). Window: Copy then Paste on the root makes
`A copy.cs` (free-name check mutation-checked); `--no-write` refuses
Paste; Copy Path fills the clipboard; Open Terminal Here starts the shell
in the folder, then a second call `cd`s it.
