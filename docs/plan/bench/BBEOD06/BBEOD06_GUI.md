# BBEOD06-B Harness timing, 2026-11-01

> Removed by headmate ruling 2026-11-01 (BBEOD06-REMOVE): backback has no MCP server ability. History only.

- One desktop fixture run printed `first_frame_ms=200` after a warm debug build. This is one observed sample, not a median.
- The in-process GPUI input and stop test took 3.42 s in its focused run. It includes a `dotnet run` of the fixture and process shutdown.
- Host load averages were 9.12, 8.48, 7.66 at first check and 4.70, 6.17, 6.81 later. The required quiet-machine condition was not met.
- After the fixture path fix, `cargo run -p bb-mcp --example bb_latency` printed 1,446 ms cold start to two tools and 174 µs warm-call p50. This was under load.
- The fixed-set median-of-five regression comparison remains unverified. Engine-only noisy measures are in `bb_BBEOD06_MCP.md`.

## BBEOD06-FIX (backback_1), noisy host

- `cargo run -p bb-mcp --example bb_latency`, load1 5.48: cold start to
  `tools/list` 1,038 ms (was 1,309-1,446 ms with `dotnet run`); warm call p50
  158 µs; Stop, 5 runs after 1 warm-up: 10/10/10/10/10 ms, median 10 ms
  (was 2,040-2,060 ms).
- 10 MB result, in-process debug test: rows built 214 ms, first paint 219 ms,
  redraw 4.7 ms (was 9.61 s and 1.78 s).
- Host was not quiet; these are noisy single-session measures.
