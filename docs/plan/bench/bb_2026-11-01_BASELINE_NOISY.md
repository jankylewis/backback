# BBEOD01 bench baseline (NOISY), 2026-11-01

## Header

- Status: **NOISY**. The quiet check (load1 < 3, no active test run, every
  live browser near 0% CPU over 10 s) never held from 01:21 to 01:35 (load1
  3.3 to 10.7). Per ruling, the baseline ran at 01:35 with 7 runs per item.
- Machine: Apple M4 Pro, 24 GB. OS: macOS 27.0.1 (26A434).
- SDK: .NET 10.0.403. roslyn-language-server 5.13.0-1.26514.10.
- App: release `bb` at 283cf3c (`lto = "fat"`, stripped).
- Command: `bb bench ../../MbTx/MbTx.slnx --runs 7` from `bb_crates/`.
- Load1: 3.61 before the run, 3.30 after.
- Live but idle: MCP browser `mbing_apexmcp` pid 39061 (19 browser
  processes, 0.3 to 0.5% CPU over 10 s); orphan `DbCores.Tests` testhost
  pid 21344 (parent launchd, 0.00 s CPU over 10 s). Active test runs: 0.
- Quiet re-baseline: first task of the next perf loop.

## Table (1 warm-up + 7 runs; median and nearest-rank p95)

| item | median | p95 | unit | load | detail |
|---|---:|---:|---|---|---|
| cold-start | 92223 | 93384 | us | 3.61>3.72 | bb doctor |
| outline-largest | 95 | 114 | us | 3.72>3.72 | HomePageSpec.cs 541 B, 3 items |
| outline-50k | 7382 | 7506 | us | 3.72>3.72 | generated 51243 B, 461 items |
| scan-harness | 240 | 290 | us | 3.72>3.72 | Harness.slnx, 4 projects |
| scan-solution | 1215 | 1305 | us | 3.72>3.72 | MbTx.slnx, 17 projects |
| audit-repo | 10983 | 11907 | us | 3.72>3.72 | 0 violations |
| binary-size | 6518784 | 6518784 | bytes | 3.72>3.72 | release build |

The OS refreshes the load average about every 5 s, so most items show one
value before and after. About 80 ms of cold-start is `dotnet --version`
(SDK time, not shell time).

## JSON lines

```json
{"item":"cold-start","unit":"us","median":92223,"p95":93384,"runs":7,"load_before":3.61,"load_after":3.72,"detail":"bb doctor"}
{"item":"outline-largest","unit":"us","median":95,"p95":114,"runs":7,"load_before":3.72,"load_after":3.72,"detail":"HomePageSpec.cs 541 B, 3 items"}
{"item":"outline-50k","unit":"us","median":7382,"p95":7506,"runs":7,"load_before":3.72,"load_after":3.72,"detail":"generated 51243 B, 461 items"}
{"item":"scan-harness","unit":"us","median":240,"p95":290,"runs":7,"load_before":3.72,"load_after":3.72,"detail":"Harness.slnx, 4 projects"}
{"item":"scan-solution","unit":"us","median":1215,"p95":1305,"runs":7,"load_before":3.72,"load_after":3.72,"detail":"MbTx.slnx, 17 projects"}
{"item":"audit-repo","unit":"us","median":10983,"p95":11907,"runs":7,"load_before":3.72,"load_after":3.72,"detail":"0 violations"}
{"item":"binary-size","unit":"bytes","median":6518784,"p95":6518784,"runs":7,"load_before":3.72,"load_after":3.72,"detail":"release build"}
```

## Perf round (one change)

- Target: audit-repo, the slowest item that is pure bb code.
- Profile (`sample`, release build with symbols), audit thread:
  `check_file` self 159 samples (inlined Rust checker); open, stat and
  getdirentries about 180; `memchr` 42; `lexer::strip` 29.
- Hypothesis: `block_lines` called `line_of` twice per `fn`/`impl`, and
  `line_of` counts newlines from the file start each time.
- Change: carry the line number in `check_blocks`; count only the newlines
  inside the block. Guard test first, green before and after:
  `block_findings_carry_the_header_line_and_the_block_length`.
- A/B in one window (01:38:42 to 01:38:58, load1 5.47 to 5.75), 3
  interleaved pairs, 7 runs each, same tree:

| pair | A baseline (us) | B after (us) |
|---|---:|---:|
| 1 | 14383 | 14929 |
| 2 | 14876 | 14988 |
| 3 | 16198 | 14325 |
| median | 14876 | 14929 |

- Result: B +0.4%, inside noise. **Not kept: reverted.** Proof: the rebuilt
  release binary is byte-identical to the baseline binary (`cmp`).
  The guard test stays (d74e5f6).
- Reading: at this repo size, folder and file syscalls cost more than
  line counting. Next candidates: one listing per folder (the dialect
  pre-scan and the walk both list every folder), or `DirEntry::file_type`
  in place of one `stat` per entry.
- Gate after the revert: fmt OK, clippy `-D warnings` OK, 83 tests passed
  0 failed, deny OK, `bb audit ..` clean.
