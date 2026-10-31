# BBEOD06-B Harness timing, 2026-11-01

- One desktop fixture run printed `first_frame_ms=200` after a warm debug build. This is one observed sample, not a median.
- The in-process GPUI input and stop test took 3.42 s in its focused run. It includes a `dotnet run` of the fixture and process shutdown.
- Host load averages were 9.12, 8.48, 7.66 at first check and 4.70, 6.17, 6.81 later. The required quiet-machine condition was not met.
- After the fixture path fix, `cargo run -p bb-mcp --example bb_latency` printed 1,446 ms cold start to two tools and 174 µs warm-call p50. This was under load.
- The fixed-set median-of-five regression comparison remains unverified. Engine-only noisy measures are in `BBEOD06_MCP.md`.
