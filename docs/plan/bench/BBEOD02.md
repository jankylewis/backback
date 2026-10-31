# BBEOD02-A performance

- Host: macOS with CommandLineTools, no full Xcode `metal` compiler.
- GPUI: Zed `ba63a02`, `gpui_platform/runtime_shaders` enabled.
- Input: `/Users/vinh.tranmodec.com/Documents/se/dna_engineering_dx/MbTx`, read-only.
- Warm-up: one run before each five-run set; runs were sequential.
- Cold process to first frame, ms: `146, 151, 152, 157, 156`; median `152`.
- RSS after 10 seconds idle, KiB: `109936, 109392, 109808, 111168, 109440`;
  median `109808` (about 107.2 MiB).
- Expand-all, 8,758 folders, ms: `635, 573, 576, 576, 574`; median `576`.
- First-frame time includes runtime shader compilation. No precompiled-shader
  build exists on this host, so isolated shader overhead is unmeasured.
- No earlier BBEOD02 baseline exists for a >5% regression comparison.
