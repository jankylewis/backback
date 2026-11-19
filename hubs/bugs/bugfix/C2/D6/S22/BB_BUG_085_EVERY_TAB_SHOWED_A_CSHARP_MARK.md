# BB-BUG-085 — fix

id: BB-BUG-085
family: UIX
status: FIXED
source: branch `fix/readback-suspects`; `lang_mark` (`bb_view/mod.rs`) + `Lang::mark`

## The change

`lang_mark` picks `Lang::of_source(path)` and `Lang::mark()` (`C#`, `RS`, `PY`); a file of no language has no mark.

## Proof

`each_tab_shows_its_own_files_language_mark` (mouse on the Explorer, Cmd+P by keys) red at b13fcbb (`[C#] x4`), green after; mutation M2 red.
