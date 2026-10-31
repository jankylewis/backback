# BBEOD02-FIX performance (B2 large file, B3 large folder)

- Host: Apple M4 Pro, macOS 27.0.1. Release build, `lto = "fat"`.
- Before: release binary built from `fba483d` (sha256 `09142ff7…`) and the
  `fba483d` crates. After: this branch, uncommitted tree at bench time.
- Load average and other `rustc` count are logged per run; no `rustc` ran.
  Load was 10.5 at run 1 and 2.3–3.0 for B3: not a certified quiet host.

## B2: 20 MB C# file (20,971,614 bytes, 1,253,233 lines, 208,872 namespaces)

Typing one char per key in the middle of the file. Five sequential runs.

| measure | before (p50 / p99 ms) | after (p50 / p99 ms) |
| --- | --- | --- |
| UI-thread work per key, run medians | 186.7 / 244.4 | 0.497 / 2.563 |
| first frame after launch with the file | n/a (no file argument) | 160 median |
| open + first parse | 2,684 on the UI thread | 2,697 on a worker |

- Before = the fba483d `Tab::reparse` path (text copy, full compare,
  `char_indices().nth`, `line_starts`, reparse), 100 keys per run.
  p99 per run: 227.7, 236.0, 255.7, 244.4, 263.3.
- After = `BB_KEYBENCH=200`: keys go through `window.dispatch_keystroke`
  (the real GPUI dispatch path). Work = dispatch time plus visible-line
  highlight time of the next frame. p99 per run: 2.517, 2.675, 2.543, 2.611,
  2.563 ms. Max per run: 3.8, 4.4, 8.6, 4.2, 4.3 ms.
- Profile on the new crates: insert 0.008 ms, tree edit 0.004 ms, 40-line
  highlight 0.40 ms, incremental tree-sitter reparse 121–128 ms even for a
  valid edit inside an expression (top-level sibling count dominates).
- Change kept: the reparse moved to a worker (`LiveTree`): 85–93 parses per
  200 keys, mean 127–138 ms each; the shown tree is edited per key.
- Target p99 <= 4 ms: met (2.563). Max over 4 ms in 4 of 5 runs: open.

## B3: folder with 10,000 files

Launch only, RSS after 10 s. Six alternating runs; run 1 is warm-up.

| measure | before median | after median | baseline (MbTx, BBEOD02-A) |
| --- | --- | --- | --- |
| first frame, ms | 439 | 197 | 152 |
| RSS, KiB | 481,696 | 114,336 | 109,808 |

- Before runs 2–6: 439, 439, 437, 457, 434 ms; 482,528 … 481,360 KiB.
- After runs 2–6: 200, 195, 195, 204, 197 ms; 113,056 … 114,592 KiB.
- After stays 45 ms above the MbTx baseline: the model reads 10,000
  entries (41 ms in the verifier harness) before the first frame.

## Open

- RSS with the 20 MB file open and edited: 1,008,592 KiB (one sample).
  No before value exists (the old build cannot open a file without a click).
