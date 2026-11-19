# BB-UIX-024 — fix

id: BB-UIX-024
family: UIX
status: FIXED
source: branch `fix/readback-suspects`

## The fix that holds

`CargoError::summary` makes the one line (tool, `file:line`, message); the section keeps the full text for a hover; the metadata error keeps stderr from its first `error` line (`failure::kept`).

## The wrong fix

Cutting the text to N characters: the place (`Cargo.toml:1`) and the message would be lost in the middle.
