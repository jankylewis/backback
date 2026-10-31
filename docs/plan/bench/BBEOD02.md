# BBEOD02-A performance

- Host: macOS with CommandLineTools, no full Xcode `metal` compiler.
- GPUI: Zed `ba63a02`, `gpui_platform/runtime_shaders` enabled.
- Input: MbTx (local checkout), read-only.
- Warm-up: one run before each five-run set; runs were sequential.
- Cold process to first frame, ms: `146, 151, 152, 157, 156`; median `152`.
- RSS after 10 seconds idle, KiB: `109936, 109392, 109808, 111168, 109440`;
  median `109808` (about 107.2 MiB).
- Expand-all, 8,758 folders, ms: `635, 573, 576, 576, 574`; median `576`.
- First-frame time includes runtime shader compilation. No precompiled-shader
  build exists on this host, so isolated shader overhead is unmeasured.
- No earlier BBEOD02 baseline exists for a >5% regression comparison.

## Reopened visual review cost

- New model hides `.git` and `.DS_Store`, and checks `.gitignore` rules when
  deciding whether to dim entries. The reachable MbTx folder count is now 8,712.
- First sequential expand-all recheck after warm-up: median 676 ms. This is
  above the prior 576 ms result. Code inspection of the folder read path found an
  extra `.gitignore` existence check on every directory.
- One change reused the directory entries already read to detect `.gitignore`.
  After a warm-up run of 747 ms, five runs were 654, 670, 664, 666, 654 ms;
  median 664 ms. It is 12 ms faster than the pre-change recheck and 15.3%
  slower than the original 576 ms benchmark. The added ignore matching is
  the functional cost; the comparison is indicative because a quiet host was
  not certified during the live readback session.
