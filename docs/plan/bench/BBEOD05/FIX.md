# BBEOD05-FIX benchmark observation (2026-11-01)

- Machine: macOS arm64 (Darwin 27.0.0), shared with other seats.
- A: `1d1b791` (exported with `git archive`); B: BBEOD05-FIX tree.
- Both: `cargo build --locked --release -p bb`, separate target dirs.
- A SHA-256: `d1649e8f9fb86245fd99480ff138adc5d99e87dd42be0b1f614ed32316229745`.
- B SHA-256: `c03e3b37ad6864dad9ed4eb2b207999dbe312343effd657e73cb934397afb9d8`.
- Command: `bb bench <MbTx root>/MbTx.slnx --runs 3` (one warm-up, three runs),
  from the B tree's `bb_crates/`, MbTx read-only. Order A,B for five pairs.
- Values: each invocation's median, microseconds.

## A/B (load1 9.75 to 10.26)

| Item | A runs | B runs | Median A | Median B | Paired delta (median, range) |
|---|---|---|---:|---:|---|
| scan-harness | 254 257 517 289 372 | 259 325 299 316 543 | 289 | 316 | +9.3% (−42.2..+46.0) |
| scan-solution | 1256 1377 2032 1387 1761 | 1294 1283 1722 1642 2563 | 1387 | 1642 | +3.0% (−15.3..+45.5) |
| audit-repo | 25494 26990 27987 27199 26095 | 26625 26646 27683 25905 30917 | 26990 | 26646 | −1.1% (−4.8..+18.5) |

## A/A control, same A binary twice (load1 10.52 to 11.36)

| Item | Paired delta (median, range) |
|---|---|
| scan-harness | −10.3% (−49.2..+52.2) |
| scan-solution | −8.1% (−47.4..+57.6) |
| audit-repo | +6.5% (−30.1..+56.1) |

## Verdict

**Noise, no regression observed; not proven.** Each A/B delta changes sign
across pairs and sits inside the A/A spread. The scan code did not change.
The audit gained one rule (`VacuousSkip`, test code only); its A/B median
moved −1.1%. The quiet condition (load1 below 3) did not hold, so a
small regression cannot be ruled out. No profile or change was made.
Re-bench on a quiet machine before a performance claim.
