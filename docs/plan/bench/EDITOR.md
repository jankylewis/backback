# BBEOD02-B editor performance

- Fixture: 5,000 lines of C#, initially 167,722 bytes, in a disposable file.
- Measure: 500 real CUA keypresses in the focused editor. GPUI histogram
  counts completed input-to-present events between the first and 500th key.
- Host was not quiet: load average was 5.18 for optimized debug and 5.47 for
  release. These runs do not certify the quiet-machine target.

| Build | Completed samples | p50 ms | p95 ms | p99 ms | RSS KiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| Debug before direct byte edit | 462 | 41.353 | 79.757 | 92.406 | 139,968 |
| Debug after direct byte edit | 377 | 33.227 | 74.252 | 124.781 | 142,816 |
| Release after direct byte edit | 207 | 33.653 | 41.189 | 41.222 | 134,880 |

- The debug before/after p99 delta is +34.9%; the change did not demonstrate a
  performance gain. The process sample during the first debug run showed 210
  samples in tab reparse and 205 in syntax reparse; only about 3 were in the
  tree-sitter parser. Direct byte-edit offsets replaced a full diff scan, but
  the live benchmark still missed 16 ms.
- Opening the unedited fixture in debug had one warm-up at 203.601 ms, then
  195.992, 195.772, 197.955, 197.032, 197.889 ms; median 197.032 ms.
  One optimized debug open was 203.180 ms. One release open was 49.684 ms.
  Five-run open medians after the change were not measured.

## Review performance loop, release build

The same 5,000-line file and release editor were used in each review run.
Instrumentation was the only code change; no behavior optimization was kept.
All rows below are real keypresses and GPUI input-to-present samples.

| Run | Key pacing | Presentations / keys | p50 ms | p95 ms | p99 ms |
| --- | --- | ---: | ---: | ---: | ---: |
| Stage probe A | Rapid | 135 / 500 | 41.124 | 41.189 | 41.222 |
| Frame trace B | Rapid | 164 / 500 | 41.091 | 41.189 | 41.419 |
| Frame trace C | 40 ms key hold | 500 / 500 | 21.823 | 21.987 | 22.036 |

- A before B: added detailed frame distribution and histogram reporting to
  the already enabled frame trace. B before C: changed only key pacing to
  isolate presentation coalescing. Neither round changed product behavior.
- C stage totals over 500 keys: input callback through buffer edit 2.487 ms;
  incremental reparse 64.988 ms; 16,533 visible-line highlight calls
  100.779 ms. Per key those are about 0.005, 0.130, and 0.202 ms.
- C GPUI mean per draw: request layout 0.172 ms, prepaint including shaping
  0.792 ms, paint 0.440 ms; platform submission 0.082 ms. Draw p50/p99 were
  1.504/1.580 ms. Dirty-to-present p50/p99 were 21.676/21.889 ms.
- All 500 frame events in B and C reported a 20.833 ms refresh interval.
  CoreGraphics reported the active display mode as 47.95 Hz. C delivered
  500 keys to 500 presentations, with p99 22.036 ms. This establishes the
  current display/vsync floor above the 16 ms budget; rapid input also coalesced
  several draws into one presentation, producing about two-refresh latency.
- A 10-second macOS `sample` capture during a rapid run found the main thread
  in `mach_msg` for 7,049 of 7,706 samples. The release binary lacked local
  symbols in the sample output. The frame trace gives the stage evidence.
- Host load was 4.41, 5.14, and 5.32 in A, B, and C. A quiet-machine gate
  remains unverified. Re-measure on a display running at least 60 Hz before
  claiming the 16 ms target on that configuration.
