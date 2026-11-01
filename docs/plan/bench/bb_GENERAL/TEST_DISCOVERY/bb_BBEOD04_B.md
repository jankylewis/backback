# BBEOD04-B test discovery timing, 2026-11-01

- Seat/branch: backback_2, `bbeod-04-gui` before commit.
- Host: Apple M4 Pro, macOS 27.0.1, .NET 10.0.403, debug `bb`.
- Fixed case: local built .NET 10 NUnit project with two `[Test]` methods.
- Command: `bb test list /tmp/bbeod04_gui_harness/Tests.csproj
  --backend dotnet`, invoked from `bb_crates/`.
- One warm-up: 979.8 ms. Five samples: 994.5, 996.2, 997.9,
  1005.6, 978.3 ms; median **996.2 ms**. Each exit was 0 with two specs.
- Load1 after the set: **10.63**. The quiet-machine condition failed.

This is an indicative current measurement. The BBEOD03-FIX VSTest median
was 975.6 ms on a different `_Harness` case, so a percent delta against it
would be unsupported. A same-case baseline and quiet-host repeat remain
open. No performance change was made from these noisy samples.
