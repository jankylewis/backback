# BB-GAT-016 — fix

id: BB-GAT-016
family: GAT
status: OBSERVED
source: `.github/workflows/bb_ci.yml`: step `Install rust-analyzer, pyright and pytest`; Tests step env `BACKBACK_REQUIRE_TOOLS: "1"`

## The fix that holds

CI installs `rustup component add rust-analyzer`, pyright 1.1.414 (npm,
own folder) and pytest 8.4.2 (a venv first on `PATH`, so the tests'
`python3 -m pytest` finds it). pyright and the venv are cached under
their own key (`lang-tools-...`), so a version bump keeps the cargo
cache. The Tests step sets `BACKBACK_REQUIRE_TOOLS=1`: a missing or
broken tool now fails the PR instead of passing on the hint.

## Proof

Local, CI's install step run as is into a scratch folder (8.6 s), its
`PATH` order, `BACKBACK_REQUIRE_TOOLS=1`: the 13 variant tests in
bb-cargo, bb-intel, bb-python and backback, 13 passed. Control: a venv
without pytest first on `PATH` turns `pytest_lists_and_runs_...` red
(`BACKBACK_REQUIRE_TOOLS is set: pytest not installed`); without the
switch it passes on the hint. The PR's own CI run is the proof that
moves this to FIXED after the merge.

## The wrong fix

Installing the tools without `BACKBACK_REQUIRE_TOOLS`: a tool that falls
off `PATH` (a cache or image change) would pass on the hint branch again,
unseen.
