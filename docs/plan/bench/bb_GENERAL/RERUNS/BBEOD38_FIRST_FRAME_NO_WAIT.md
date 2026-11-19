# BBEOD38 — the first frame never waits for the language walk, 2026-11-20

Apple M4 Pro (12 cores), macOS 27. Release bundles (`bb_bundle.sh`):
`aa1d223` (base, before the picker), the BBEOD36 fix bundle (BB-PRF-007,
the code merged as `708d624`; plist label `19dd6d3`), and this fix
(BB-PRF-009; built from the uncommitted worktree on `243c495`, so its plist
label reads `243c495`; its binary holds the new "Detecting languages…"
string). Same method as BBEOD36/37: the bundle binary exec'd with a
Dock-like environment (cwd `/`, minimal PATH, no tty, own session),
`--no-write`, an own empty state folder per run, own pid stopped by TERM.
"First frame" = exec to the app's `first_frame_ms` line; the app's own
number in brackets. Per case 1 warm-up + 5, builds interleaved run by run;
two full matrices (03:47-03:49 and 03:50-03:53), so 10 samples a case.
Load average 2.6-7.2 on 12 cores: NOT a quiet machine (other fleets).
72 + 72 launches: 0 timeouts, 0 apps or children left.

## First frame, medians of 10 (verdict >5 % vs base = regression)

| Case | base `aa1d223` | BBEOD36 | this fix | fix vs base |
|---|---:|---:|---:|---|
| dna-ax, first open (no saved choice) | 184 (55) ms | 341 (217) | 189 (57) | +2.7 % ok |
| dna-ax, saved choice C#+Python | 179 (55) | 179 (57) | 183 (57) | +2.5 % ok |
| MbTx folder | 186 (58) | 196 (71) | 187 (59) | +0.5 % ok |
| 50,000 `.cs` files | 187 (58) | 189 (60) | 189 (59) | +1.3 % ok |

Per matrix (medians of 5): dna-ax first open 179 / 339 / 189 (+5.6 %;
the fix's samples ran at load 2.7-7.1) and 184 / 343 / 189 (+2.7 %). The
app's own number is 55 vs 56 and 56 vs 57 in those runs: the +10 ms of
run 1 is outside the app (exec and load); not proven either way, so the
10-sample median above is the verdict.

App RSS 1 s after first frame: 101.4-103.3 MB, all builds alike.

## What changed

`build_window` no longer joins the launch's walk. When the walk has not
ended, the window opens with no language active ("Detecting languages…"
in the status bar; F5 says Debug is detecting) and polls the walk's
answer every 4 ms on a gpui timer; on the answer it saves a complete
walk (as before), sets the menus, starts the servers and Cargo rows,
recolours open tabs and sends them to the servers that started. A walk
that already ended, the Start Page (ticks chosen) and File > Open paths
resolve as before.

## Gate

`gate.sh --full` 03:55-04:03 (1 worker): fmt, clippy, test, deny, sweep,
audit rc=0; suites=67 passed=737 failed=0; `gate: green`.
