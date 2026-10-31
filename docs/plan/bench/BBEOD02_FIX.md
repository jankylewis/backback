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

## Memory with the 20 MB file open (branch bbeod-02-mem)

Heap profile (`heap`, library harness, same file):

| stage | live heap | malloc nodes | RSS |
| --- | --- | --- | --- |
| file opened (rope) | 23.8 MB | 23,535 | 54 MB |
| full tree-sitter parse | 886.3 MB | 8,796,177 | 883 MB |
| tree dropped | 24.1 MB | 23,551 | 859 MB |

- Biggest holder: the syntax tree, 862 MB (97% of heap): 6,892,780 x 96 B,
  1,253,232 x 112 B (one per line), 417,746 x 80 B, 208,874 x 128 B. About
  41 tree bytes per source byte. Freed pages also stay in RSS.
- One change: files over 4 MiB parse only the lines around the view (+-1,000
  lines, tree-sitter included ranges, absolute offsets kept); the window
  follows scrolling. Smaller files are unchanged.
- A = e45b4d2 release, B = change. Six alternating runs, run 1 warm-up,
  medians of runs 2-6; 200 keys mid-file via `BB_KEYBENCH`:

| measure | A | B | change |
| --- | --- | --- | --- |
| RSS after open (8 s), KiB | 1,008,144 | 165,904 | -83.5% |
| RSS after 200 keys, KiB | 1,008,112 | 165,904 | -83.5% |
| live heap after 200 keys, bytes | 905,310,529 | 43,950,929 | -95.1% |
| open (load + parse), ms | 2,706 | 95.7 | -96.5% |
| UI work per key p99, us | 2,466 | 230 | -90.7% |

- Keys mid-file fall outside B's window (the view stays at the top), so 3
  more runs typed 200 keys at line 1, inside the window: A p99 2,435-2,540
  us, RSS ~1,008 MB; B p99 230-236 us, RSS 166-168 MB, parse < 1 ms per key.
- Load was 4.2-17.8 (logged per run); the RSS gap is far above the noise.
- Kept: smaller and faster, gate green.

## Open

- Large-file mode: lines outside the window show plain until the next parse;
  a construct that starts above the window can colour its first lines wrong.
