# BBEOD03-FIX2 readback, 2026-11-01

## B1: Apex list loses the MSBuild import chain

- Family: a project evaluated outside its real folder loses the files MSBuild
  imports by folder (`Directory.Build.props`, `Directory.Packages.props`,
  `global.json`, `nuget.config`).
- Repro: on MbTx, `bb test list MbTx.slnx` failed 8/11 projects with
  `Could not resolve TargetPath`. The shadow copy had only the `.csproj` and
  `bin/`; MbTx sets `TargetFramework` in `Directory.Build.props`.
- Why the fixture missed it: every harness project set the framework inline.
- Failing-first: `_NUnitApex` now takes its framework from its own
  `Directory.Build.props`. `list_leaves_the_built_solution_tree_unchanged`
  then failed with the same MbTx error. (The `_Harness` root already holds 5
  entries, so the file sits in the project folder.)
- Fix: evaluate `TargetPath` of the real project in place
  (`msbuild -getProperty`, read-only), copy only the build output to temp, and
  hand the Apex CLI a stub project with that copied `TargetPath`. The stub has
  no SDK, so it imports nothing. The CLI's reported assembly must equal the
  copy, else the list fails.
- Guard: `msbuild_reads_the_stub_target_path_back_unchanged` runs real MSBuild
  on a path with `$ @ % ; ' ? * & < >`. With escaping removed it fails.
- Family sweep: `shadow::project` was the only place that evaluated a project
  outside its folder. It is removed. VSTest and MTP already evaluate in place.

| MbTx, read-only | Result | `find -newer` |
|---|---|---|
| `bb test list` default | 11/11 projects, exit 0 | 0 files |
| `bb test list --backend dotnet` | 11/11 projects, exit 0 | 0 files |

Spec counts are equal per project on both backends (857, 472, 68, 250, 123,
22, 40, 15, 141, 591, 316).

## O1: a dead server leaves its squiggles

- Family: server-owned UI state survives when the server is gone.
- Instances: `Event::Stopped` cleared questions and popups, but kept the
  diagnostics. `Event::Failed` and a failed send to the worker set only the
  status: questions, popups and diagnostics stayed.
- Failing-first: three window tests, one per path, failed on the diagnostics.
- Guard: one `server_gone(status)` method clears questions, popups,
  diagnostics (squiggles and Problems), references, jump and rename. All three
  paths call it. Restart pulls fresh diagnostics from the new server.
