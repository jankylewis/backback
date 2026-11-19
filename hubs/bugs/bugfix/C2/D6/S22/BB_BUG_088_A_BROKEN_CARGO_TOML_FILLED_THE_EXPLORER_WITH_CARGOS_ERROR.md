# BB-BUG-088 — fix

id: BB-BUG-088
family: UIX
status: FIXED
source: branch `fix/readback-suspects`; `CargoError::summary`, `bb_rows/cargo.rs` `failure`

## The change

`CargoError::summary` (tool, `file:line` from cargo's `-->` line, first `error:` message); `CargoMembers::Failed { summary, full }`; the Explorer line is one non-wrapping row with `full` as its tooltip; the Tests panel's metadata errors use the summary; `members` keeps stderr from its first `error` line.

## Proof

`a_broken_cargo_toml_shows_one_short_line_and_the_full_text_on_hover` red at b13fcbb (section 176 px), green after; `bb_cargo` `failure::tests` (3); mutation M5 red.
