# BBEOD08-REAP benchmark, 2026-11-01

- Release `bb`, one warm-up and five runs per item on `MbTx.slnx`.
- Quiet condition failed: load1 was 7.08 throughout this run, above the
  required 3. Other fleet test work was active. The earlier baseline was
  also marked noisy, at load1 3.61. No performance verdict is supported.
- Process ownership was the required correctness change. No speculative
  performance change was kept.

| Item | Baseline median | This median | Unit |
|---|---:|---:|---|
| Cold start | 92223 | 98281 | us |
| Outline 50k | 7382 | 7354 | us |
| Scan Harness | 240 | 272 | us |
| Scan solution | 1215 | 1233 | us |
| Audit repo | 10983 | 59551 | us |
| Binary size | 6518784 | 8620800 | bytes |

The baseline used seven runs and an older tree. The current largest C# file
also differs, so its outline result is excluded from the comparison. These
raw values are evidence only; the quiet-machine regression check remains
unverified.
