# BBEOD36 — the language work against the C# app before it, 2026-11-20

Apple M4 Pro (12 cores), macOS 27. Release bundles (`bb_bundle.sh`) of
`aa1d223` (before the picker), master `19dd6d3`, and the fix (BB-PRF-007).
Each case: 1 warm-up, then 5 samples, median; builds interleaved run by run.
Load average 2.2-3.6 (12 cores; not idle: other fleets at work). Every run had
its own empty state folder and stopped only its own pid; no process left.

Launch: the bundle's binary started with a Dock-like environment (cwd `/`,
minimal PATH, no terminal, own session). Real `open` (LaunchServices)
launches of these unsigned scratch bundles hung on folders under the user's
Documents for both builds (main thread in `open(2)`, 0 % CPU; a privacy
prompt is the likely cause, not proven); they work on scratch folders, so
the `open` rows use a scratch tree. "First frame" = exec to the app's
`first_frame_ms` line; the app's own number is in brackets.

## C#: before / after (verdict >5 % = regression)

| Case | `aa1d223` | `19dd6d3` | fix | master vs base | fix vs base |
|---|---:|---:|---:|---|---|
| First frame, MbTx folder (6,673 files) | 204 (67) ms | 328 (193) | 213 (69) | +61 % REGRESSION | +4 % ok |
| First frame, 50,000 `.cs` files | 191 (62) | 274 (146) | 193 (61) | +43 % REGRESSION | +1 % ok |
| Same, through `open` | 244 (61) | 322 (136) | 242 (63) | +32 % REGRESSION | -1 % ok |
| Apex, no saved choice: first frame | 193 (61) | 205 (76) | 188 (64) | +6 % REGRESSION | -3 % ok |
| Apex: open to Roslyn Ready | 7,139 ms | 7,276 | 7,240 | +2 % | +1 % ok |
| Apex, C# ticked: first frame | 187 | 189 | 190 | +1 % | +2 % ok |
| Apex, C# ticked: open to Roslyn Ready | 7,397 | 7,291 | 7,466 | -1 % | +1 % ok |
| App RSS 10 s after Ready (Apex) | 103.1 MB | 103.8 | 103.6 | +0.6 % | +0.5 % ok |
| Roslyn RSS 10 s after Ready, C# ticked | 363.6 MB | 362.8 | 362.0 | 0 % | 0 % ok |
| Keys: 300 chars in a 1,768-line `.cs`, UI work p50 / p99 | 1.95 / 7.50 ms | 1.96 / 7.31 | 1.94 / 7.53 | 0 % | 0 % ok |
| Binary size | 14.23 MB | 16.05 | 16.05 | +12.8 % | +12.8 % (see below) |

A first run of the matrix (same day, load 2.7-3.5) gave the same picture:
MbTx 187 -> 325 ms, 50,000 files 182 -> 272 ms, C# ticked 189 -> 190 ms.
Roslyn Ready is measured on Apex: on MbTx it stays at "Loading 22/28
projects" for minutes on both builds (restore is off), so no Ready time.

## Not equal, by design (for a ruling, not changed here)

- A C# solution that also holds Python files (Apex: 5, MbTx: 691) opens with
  C# and Python ticked when no choice is saved, so pyright starts too:
  children RSS on Apex 363 -> 494-511 MB, MbTx 822 -> 1,022 MB. With C#
  ticked it is the same as before. The picker's default (BBEOD26) decides it.
- The binary grew 1.8 MB (+12.8 %): the Rust and Python code. Not profiled.

## detect() on real trees (400 ms budget)

`bb_lang::detect`, release, warm cache, 7 runs, median (C#-only walk):

| Tree | Found | Median | Max | Until C# |
|---|---|---:|---:|---:|
| MbTx | C#, Python | 113 ms | 163 | 0.7 ms |
| Apex | C#, Python | 9.6 | 17.9 | 0.8 |
| The dna-ax repo | C#, Python | 266 | 292 | 5.6 |
| backback | C#, Rust | 17.3 | 24.2 | 1.2 |
| 50,000 `.cs` files | C# | 74 | 85 | 0.5 |
| The whole work folder (all three, stops early) | all | 19 | 35 | 7.0 |

None reached the deadline; the closest is 266 ms (73 % of it), a C#
folder with no Rust, which must read to the end. Cold cache not measured.

## Rust-only and Python-only baselines (master, new)

Scratch fixtures: a `cargo` package with a 1,621-line `.rs`; a
`pyproject.toml` package with a 1,516-line `.py`. "Ready" = initialized.

| Case | Rust | Python |
|---|---:|---:|
| First frame (app's own) | 184 (58) ms | 182 (58) ms |
| Same, through `open` | 237 (62) | 247 (61) |
| Open to server Ready | 300 ms (rust-analyzer) | 510 ms (pyright) |
| App RSS / server RSS, 10 s after Ready | 102.3 / 603 MB | 102.1 / 88 MB |
| Keys p50 / p99 | 1.77 / 2.06 ms | 1.85 / 5.63 ms |

The fix leaves them equal (fix vs master: first frame 188 vs 188 and
187 vs 191 ms; Ready 309 vs 413 and 512 vs 520 ms).

## Change kept and gate

One change: the walk starts on its own thread at launch (BB-PRF-007).
Profile evidence: the master-only first-frame growth equals `detect` time
on the same tree (113 ms MbTx, 74 ms 50,000 files). Gate counts: the PR.
