# Roadmap: macOS and C# only

Source of truth: `AGENTS.md` and the AgentFarm `backback` goals board. Each BBEOD
closes one milestone with a demonstrated exit result.

| Unit | Milestone | Exit result |
|---|---|---|
| BBEOD01 | M0 Engine proof | `bb` doctor, scan, open, outline, audit, bench; fixture e2e; Roslyn loads MbTx with time recorded; first readback and benchmark; gate green. |
| BBEOD02 | M1 Editor | Native GPUI/Metal window; dark default and light; solution tree, tabs, edit/save, tree-sitter; typing p99 ≤ 16 ms on 5k lines. |
| BBEOD03 | M2 Code intelligence | Roslyn completion, diagnostics, definition, references, rename, fixes; no crashes on MbTx, Apex, NmcTx, SI readback. |
| BBEOD04 | M3 Test automation | NUnit/xUnit/MSTest on VSTest and MTP; one/class/category; runsettings; Playwright traces; Apex reports; ApexTestkit parity on MbTx. |
| BBEOD05 | M4 Debugging | netcoredbg breakpoints, step, locals, stack, watch, one-test debug, set-next-statement; one MbTx spec debugged end to end. |
| BBEOD06 | M5 Harness | MCP harness removed by headmate ruling 2026-11-01. Terminal and find in files: not built; propose before building. |
| BBEOD07 | M6 Open-source v1 | Signed, notarised macOS app; original logo, Homebrew cask, docs, CONTRIBUTING, published benchmarks, v1.0 release. |

## Seven test layers

1. Pure unit tests for text, syntax, paths, and protocol values.
2. File and codec tests for persisted and wire formats.
3. CLI tests for `bb` commands and their exit results.
4. Process tests against `dotnet`, Roslyn, and netcoredbg.
5. Fixture e2e tests through the served app surface.
6. Readback on real solutions, read-only where `AGENTS.md` requires.
7. Performance benchmarks with a fixed case set and a baseline.

These layers are the plan's test map. Each milestone adds the relevant live
checks. A green lower layer does not prove a higher layer.

## Every BBEOD

Readback: use backback on fixtures and real solutions; log an issue, write a
failing test at the defect's layer, fix, run the gate, repeat the same user
steps, and record the lesson in `readback/`.

Performance: run a fixed benchmark set on a quiet machine. Take the median of
five after warm-up, compare with baseline, and treat a rise above 5% as a
regression. Profile, make one change, re-benchmark, and keep it only when faster
with all tests green. Record the result in `bench/`.

## BBEOD05 risk

The inspected netcoredbg revision lacks DAP `gotoTargets`/`goto` and CoreCLR
`SetIP` use. Set-next-statement needs an upstream change estimated at 1–2
engineering weeks plus cross-platform validation. This estimate is research,
not a live debugger result. See `../../rnd/SETNEXT.md`.
