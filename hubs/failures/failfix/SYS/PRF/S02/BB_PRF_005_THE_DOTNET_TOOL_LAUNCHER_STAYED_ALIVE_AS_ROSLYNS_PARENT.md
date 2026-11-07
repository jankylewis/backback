# BB-PRF-005 — fix

id: BB-PRF-005
family: PRF
status: FIXED
source: `bb_crates/edge/bb_lsp/src/roslyn/bb_direct.rs` (`command`, `server_behind`)

## The fix that holds

Start the server behind the launcher: `<tools>/.store/<name>/<version>/
<name>.<rid>/<version>/tools/<tfm>/<rid>/Microsoft.CodeAnalysis.LanguageServer`,
only when exactly one exists; else the launcher, as before. `DOTNET_ROOT`
is set from the `dotnet` on `PATH` when the user set none (the launcher
told the server where .NET is). Same arguments and environment otherwise.

## Wrong fixes

- GC tuning (`DOTNET_GCConserveMemory` 5 or 9, `DOTNET_TieredPGO=0`): 3
  runs each on MbTx, idle Roslyn 259-554 MB in every setting, inside the
  run-to-run spread of the default (339-526 MB): no evidence, not shipped.
- Picking the newest of several store versions: the launcher's choice is
  not known from the folders; two versions keep the launcher.

## Proof

MbTx, 3 runs: the tree is backback + the server (was + launcher); 17 of
17 projects load; ready 6.4-8.7 s (was 6.7-9.2 s). `bb probe` hover x10:
load 6.62 s (6.75), first 172 ms (175), warm p50 1.07 ms (1.17), same
text. Unit tests: one server is used; none or two keep the launcher.
