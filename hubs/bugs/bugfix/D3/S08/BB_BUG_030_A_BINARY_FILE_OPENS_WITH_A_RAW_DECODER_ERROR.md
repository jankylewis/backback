# BB-BUG-030 — fix

id: BB-BUG-030
family: UIX
status: FIXED
source: `bb_crates/bb_core/bb_text/src/document/mod.rs` (`DocumentError::Utf8`)

## The change

The message reads `binary or non-UTF-8 file: not shown as text`; the
decoder's detail stays in `source()`.

## Proof

Readback (branch build): the `.pyc` launch prints `tab=None` and that
message; `bb_text` tests 50 passed.
