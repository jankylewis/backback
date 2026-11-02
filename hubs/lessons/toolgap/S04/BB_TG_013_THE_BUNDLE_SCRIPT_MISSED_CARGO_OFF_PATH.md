# BB-TG-013 — The bundle script missed cargo off PATH

id: BB-TG-013
for: none
status: FIXED
date: 2026-11-03
source: `install.sh` from an agent shell without rustup's folder on `PATH`

`bb_bundle.sh` read the version with `cargo metadata | python3 ...`. With
no `cargo` on `PATH`, the pipe fed Python an empty string and the run ended
in a `JSONDecodeError` trace, not a missing-tool message. `install.sh` had
not touched `/Applications` yet (it stops under `set -e`), but the cause
was not readable. `gate.sh` already adds `$HOME/.cargo/bin` (BB-TG-002).

Fix: `bb_bundle.sh` adds `$HOME/.cargo/bin` to `PATH` like `gate.sh`, and
stops with `refused: cargo not found ...` (exit 2) when there is still no
`cargo`.

Proof: before, `env -i HOME PATH=/usr/bin:/bin bash bb_bundle.sh` gave the
trace. After, a home with no cargo gives the refusal and exit 2, and the
real home with launchd's `PATH` builds and signs the bundle.
