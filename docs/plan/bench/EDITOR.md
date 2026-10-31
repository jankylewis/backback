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
- The p99 target and the quiet-host gate remain unresolved. The release result
  is not comparable with the debug baseline as an optimization delta.
