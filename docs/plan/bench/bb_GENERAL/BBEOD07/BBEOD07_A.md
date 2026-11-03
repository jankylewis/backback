# BBEOD07-A bench, 2026-11-01

- Command: bundled release `bb bench MbTx.slnx --runs 5` from `bb_crates/`.
- One warm-up and five samples per item. Machine load was 3.24 during the run.
- The load missed the quiet threshold (<3), so this run is noisy evidence.
- The BBEOD01 baseline is also marked noisy. No valid regression verdict follows.

| item | median | BBEOD01 noisy median | change |
|---|---:|---:|---:|
| cold-start | 92357 us | 92223 us | +0.1% |
| outline-largest | 99 us | 95 us | +4.2% |
| outline-50k | 7359 us | 7382 us | -0.3% |
| scan-harness | 235 us | 240 us | -2.1% |
| scan-solution | 1234 us | 1215 us | +1.6% |
| audit-repo | 11211 us | 10983 us | +2.1% |
| binary-size | 6481008 B | 6518784 B | -0.6% |

The benchmark reported 0 audit violations. This release task changed no Rust
source. A quiet baseline and quiet rerun remain open.
