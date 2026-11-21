# BB-PRC-010 — fix

id: BB-PRC-010
family: PRC
status: FIXED
source: branch `bbeod-43-no-compiler-server`, `edge/dotnet/bb_sdk/src/sdk/bb_no_servers.rs`

## The fix that holds

One helper, `bb_dotnet::no_servers`. `command()` is `dotnet` as the leader of
its own process group with `MSBUILDDISABLENODEREUSE=1` and
`UseSharedCompilation=false` in its environment: every MSBuild under it reads
them, so `dotnet test` builds and the Apex CLI's inner build are covered too.
`run()` waits, then ends whatever the group still holds (TERM, 0.5 s, KILL),
never a process outside it. All 10 places use it; Build Solution and the
debugger's build also keep `FLAGS` on their command line; the test engine's
`Call` sets `ENV` on every tool it runs. A guard test fails on any other
`Command::new("dotnet")` in the workspace.

## The wrong fix

Ending `VBCSCompiler` by name after the tests: the user's VS Code or Rider
server has the same name and pipe. Flags on `dotnet build` only: `dotnet test`
and the Apex CLI build without them. `-p:CscToolPath=`: turns the server off
"by design" but swaps the compiler path.

## Proof

RED at a9053fd (helper = the suite's old call): both compiles `server
processed compilation`. GREEN: `by design` twice, nothing left in the build's
group. Mutants: 6 of 6 killed, one at a time (fleet `evidence/BBEOD43_READY.md`).
