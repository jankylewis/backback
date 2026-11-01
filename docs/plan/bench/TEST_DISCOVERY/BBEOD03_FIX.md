# BBEOD03-FIX test discovery benchmark, 2026-11-01

- Host: Mac16,8, macOS 27.0.1. Debug `bb` binaries.
- Fixed input: built `_Harness` copy, NUnit VSTest and Apex, xUnit MTP.
- Baseline: `908d247`; changed tree: BBEOD03-FIX before commit.
- One warm-up per binary and backend; five alternating runs per binary.
- Host load was 5.00 at the later check. The quiet-machine gate was not met,
  so these times are indicative and require a quiet-host repeat.

| Backend | Baseline samples, ms | Baseline median | First fix samples, ms | First median | Delta |
|---|---|---:|---|---:|---:|
| VSTest | 910.5, 908.9, 924.9, 916.5, 944.3 | 916.5 | 955.2, 976.5, 975.6, 973.3, 977.3 | 975.6 | +6.4% |
| MTP | 526.2, 531.2, 528.6, 541.9, 531.9 | 531.2 | 586.8, 580.9, 578.2, 576.2, 582.2 | 580.9 | +9.4% |
| Apex | 1061.4, 988.7, 1079.5, 1089.2, 1035.0 | 1061.4 | 1349.6, 1355.6, 1545.4, 1376.5, 1365.8 | 1365.8 | +28.7% |

The first Apex implementation made an extra MSBuild `TargetPath` query. I
removed that query and mapped the shadow assembly path back to its source.
The copy now includes only runtime assets. This is the one measured change.

| Backend | Baseline repeat, ms | Median | Revised samples, ms | Median | Delta |
|---|---|---:|---|---:|---:|
| Apex | 1053.1, 1051.5, 1001.2, 1007.8, 1006.1 | 1007.8 | 1077.4, 1052.3, 1038.6, 1050.9, 1052.9 | 1052.3 | +4.4% |

The test output folders held 37 files / 8.3 MiB for NUnit and 26 files /
123 MiB for xUnit. Copy cost is a likely cause of the VSTest and MTP deltas;
this is an inference from file size, not a CPU profile. The safety fix stays
because discovery changed the user's files before it. Quiet-host profiling
and repeat timing remain open.
