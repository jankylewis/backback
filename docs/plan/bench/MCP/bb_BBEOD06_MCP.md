# BBEOD06-A MCP and fixed-set benchmark, 2026-11-01

> Removed by headmate ruling 2026-11-01 (BBEOD06-REMOVE): backback has no MCP server ability. History only.

- Machine: Apple M4 Pro, macOS 27.0.1.
- Quiet check: failed. Load1 was 3.39 before MCP timing, 6.09 on the first fixed-set run, and 11.28 on the final tree.
- MCP fixture: one warm-up, then five `Greet` calls. Final cold start through `tools/list`: 1,309 ms. Warm call p50: 135 µs.
- Earlier diagnostic run before log-stream fix: 1,650 ms cold, 172 µs warm p50.
- No earlier MCP baseline exists, so no MCP regression percentage can be verified.
- Fixed set: release `bb bench <MbTx>/MbTx.slnx --runs 5`.
- Read-only scan of MbTx; no real MCP server started.

| Item | BBEOD01 noisy baseline | BBEOD06 noisy median | Delta |
|---|---:|---:|---:|
| Cold start | 92,223 µs | 93,929 µs | +1.8% |
| Outline 50k | 7,382 µs | 7,372 µs | -0.1% |
| Scan harness | 240 µs | 240 µs | 0% |
| Scan MbTx | 1,215 µs | 1,433 µs | +17.9% |
| Audit repo | 10,983 µs | 13,451 µs | +22.5% |

The BBEOD01 baseline was also marked noisy. These deltas are diagnostic only.
The first BBEOD06 fixed-set run at load1 6.09 measured scan MbTx 1,281 µs and audit 12,142 µs.
The current machine load fails the quiet benchmark condition. A quiet re-run is still required.
