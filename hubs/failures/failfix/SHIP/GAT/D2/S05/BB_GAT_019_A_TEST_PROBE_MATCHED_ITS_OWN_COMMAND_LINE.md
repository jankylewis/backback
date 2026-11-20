# BB-GAT-019 — fix

id: BB-GAT-019
family: GAT
status: FIXED
source: branch `bbeod-41-python-opt-in`, the commit after fce43aa

## The fix that holds

The probe (`pyright_servers` in `lang_tests/tools/python/mod.rs`) skips its own pid and counts a row only when the program is a pyright server: the first word, or the script `node` runs (`node .../pyright-langserver --stdio`), is named in `bb_python::pyright::SERVERS`. Arguments never count. The window test's control still sees the real `pyright-langserver` once Python is ticked; `the_probe_sees_a_pyright_server_and_never_a_test_named_for_pyright` feeds it the CI row.

## The wrong fix

Renaming the test so its name lacks `pyright`; running it under `cargo test` only; filtering out `--exact`; skipping the probe on CI.

## The proof

Same nextest command: red on fce43aa (the test's own row), green after (2 passed, control `node /opt/homebrew/bin/pyright-langserver --stdio`).
