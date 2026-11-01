# BBEOD07-FIX bench, 2026-11-01

- Command: release `bb bench ../../../MbTx/MbTx.slnx --runs 5` from `bb_crates/`.
- One warm-up and five samples per item. Load was 9.41 throughout.
- The quiet threshold is below 3. This run is noisy, as was the BBEOD01 baseline.
- No valid regression verdict follows. This fix changed no Rust source.

| item | median | unit |
|---|---:|---|
| cold-start | 90721 | us |
| outline-largest | 100 | us |
| outline-50k | 7351 | us |
| scan-harness | 267 | us |
| scan-solution | 1203 | us |
| audit-repo | 48676 | us |
| binary-size | 8554304 | bytes |

The benchmark reported 0 audit violations and 17 solution projects.
The quiet baseline and a quiet comparison remain open.
