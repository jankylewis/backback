# BB-CIP-002 — fix

id: BB-CIP-002
family: CIP
status: FIXED
source: `.github/workflows/bb_ci.yml` (`runs-on: macos-latest`)

## The fix that holds

`runs-on: macos-latest`, the label every other run here uses (arm64).

## Proof

Run 41325098190 started within seconds of the push.
