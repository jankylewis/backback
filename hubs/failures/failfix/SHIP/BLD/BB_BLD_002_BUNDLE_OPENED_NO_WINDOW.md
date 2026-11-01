# BB-BLD-002 — fix

id: BB-BLD-002
family: BLD
status: FIXED
source: commit `2151199`; `docs/release/scripts/verify_bundle.sh`

## The fix that holds

The GUI binary is `CFBundleExecutable`; `verify_bundle.sh` checks the served
app, not a first-frame signal.

## The wrong fix

Trusting a first-frame log line as proof that a window exists.

## Proof

An independent mate opened the rebuilt `.app` and saw the window; PR #7 merged.
