# BBEOD06 Find in Files bench, 2026-11-03

By backback_mm, branch `bbeod-06-find`. Engine `bb-search`, release build,
`cargo run --release -p bb-search --example find_bench <root> <query> 5`: one
warm-up run, then 5 timed runs, median. Root: the Apex solution folder
(read-only; 1,236 text files read after skips and `.gitignore`). Host not
quiet: load average 6.8-7.9 on both sides.

| Query | Hits | Median before | Median after |
|---|---|---|---|
| `ToolOutcome` (case-sensitive) | 141 | 161 ms | 43 ms |
| `locator` (smart case; capped) | 2,000 | 4 ms | 2 ms |
| `/async\s+Task<\w+>/` | 265 | 95 ms | 43 ms |
| `zzzNoSuchTextzzz` (no hit) | 0 | 60 ms | 42 ms |

Change kept: test the whole file once and skip its line loop when nothing
matches. The regex runs with `multi_line` and `crlf`, so `^` and `$` match
at every line end and a matching line is never skipped (guard test
`anchors_match_at_every_line_end_including_crlf_files`, CRLF and LF files).
Hit counts are equal before and after. `git grep` on the same tree finds
the same 141 `ToolOutcome` lines.

Not met: the quiet-machine rule. Before and after ran back to back under
the same load; spread within each run of 5 is up to 35%.

## Addendum 2026-11-03: search after idle, in the app

Same query in the app after 10 s idle, 5 times per build: master 259 / 77 /
433 / 679 / 480 ms (load 13.8), later 317 / 215 / 176 / 370 / 291 ms (load
7.7); with the search at `gpui::Priority::High`: 357 / 669 / 624 / 746 /
446 ms (load 13.7). No gain shown, so the change was dropped. The spread
follows CPU load (12 cores: 8 P + 4 E, load 8-14 from other work), not
memory (76% free).

