# BBEOD01-B: Apex Testkit compatibility contract

Source: AgentFarm `state/fleet/backback/evidence/SPIKE_APEXTESTKIT.md`.

Source key: `Apex@1a0e7b2` means `git show 1a0e7b2:<path>` in the read-only Apex repository. The extension is absent from this checkout's current `HEAD`; commit `1a0e7b2` contains the extension and matching IDE-mode CLI. MbTx pins `apex.cli` `0.8.2-pre`, and its installed `dotnet apex --help` reports the IDE flags. The current Apex branch's CLI source is older and lacks those modes. Sources: `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/package.json:1-17`, `MbTx/.config/dotnet-tools.json:1-11`, `Apex/ApexToolhub/Apex.Cli/Program.cs:16-22`, `Apex@1a0e7b2:ApexToolhub/Apex.Cli/Program.cs:15-22`.

## Discovery

- The extension activates for workspaces containing `.sln` or `.slnx`. It finds `.csproj` files, excluding `bin`, `obj`, and `node_modules`, and accepts projects containing `Microsoft.NET.Test.Sdk` or `NUnit3TestAdapter`. An optional `solution` setting limits projects to paths parsed from the solution. Sources: `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/package.json:13-16`, `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/_testkit.ts:38-70`.
- Discovery is lazy when a project expands. Invoke `<command> test --project <absolute-csproj> -c <configuration> --list --json --no-build` with the workspace folder as cwd. Nonzero exit fails discovery. Sources: `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/_testkit.ts:73-79`, `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/discovery.ts:54-58`, `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/runner.ts:13-17`.
- Stdout must be one JSON object: `{ "project": string, "assembly": string, "specs": [{ "fqn": string }] }`. Each FQN has namespace, class, and method components; the UI groups Project > Namespace > Class > Spec. Sources: `Apex@1a0e7b2:ApexToolhub/Apex.Cli/SpecListJson.cs:15-37`, `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/discovery.ts:3-36`, `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/_testkit.ts:80-100`.
- The matching CLI's list path optionally builds, resolves `TargetPath` through MSBuild, runs `dotnet vstest <assembly> --ListFullyQualifiedTests --ListTestsTargetPath:<file>`, and emits either JSON or one FQN per line. Sources: `Apex@1a0e7b2:ApexToolhub/Apex.Cli/ListCommand.cs:5-32`, `Apex@1a0e7b2:ApexToolhub/Apex.Cli/ListArgvBuilder.cs:8-33`.
- The extension links class and spec items to C# source by scanning project `.cs` files for matching class and method names. Sources: `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/_testkit.ts:79-99`, `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/discovery.ts:39-52`.

## Run and debug

- A run selects leaf FQNs, excludes requested leaves, groups by project, and marks them started. Before each project run, `buildBeforeRun=true` runs `dotnet build <csproj> -c <configuration>`; build failure errors the leaves. Sources: `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/_testkit.ts:133-165`.
- Invoke `<command> test --project <csproj> -c <configuration> --specs <comma-separated-FQNs> [--settings <path>] --trx <temporary-run.trx> --no-build`. A relative runsettings path resolves from the workspace folder. The command prefix is split on whitespace and spawned without a shell; quoted paths in that setting are not parsed as shell quotes. Sources: `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/_testkit.ts:154-173`, `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/runner.ts:7-17`.
- `--specs` overrides `--topic`; the matching CLI builds `FullyQualifiedName=...` filters, forwards `--settings`, and adds a TRX logger to `dotnet test`. Sources: `Apex@1a0e7b2:ApexToolhub/Apex.Cli/TestCommandParser.cs:28-36`, `Apex@1a0e7b2:ApexToolhub/Apex.Cli/TestArgvBuilder.cs:14-28`, `Apex@1a0e7b2:ApexToolhub/Apex.Cli/TestArgvBuilder.cs:44-77`, `Apex@1a0e7b2:ApexToolhub/Apex.Cli/SpecFilter.cs:30-50`.
- Debug adds `--debug`. The CLI sets `VSTEST_HOST_DEBUG=1`, parses a test-host process ID, and emits a line `{"testHostPid":N}`. The extension attaches a `coreclr` session with `processId` as a string and `justMyCode:false`; it errors when the PID line is absent or attach fails. Sources: `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/debugger.ts:4-23`, `Apex@1a0e7b2:ApexToolhub/Apex.Cli/TestArgvBuilder.cs:31-42`, `Apex@1a0e7b2:ApexToolhub/Apex.Cli/RunCommand.cs:7-36`, `Apex@1a0e7b2:ApexToolhub/Apex.Cli/HostPidParser.cs:8-31`.

## Result contract

- Read the produced TRX, require `TestRun/Results`, map `UnitTestResult.testId` through `TestDefinitions/UnitTest/TestMethod.className + name` to the FQN, and read outcome, duration, error message, and stack trace. `Passed` passes; `NotExecuted`/`Inconclusive` skip; `Failed` fails; unknown outcomes error. Sources: `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/runner.ts:35-75`, `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/tooling/tests/fixtures/run.trx:3-11`.
- A selected spec absent from TRX errors that leaf. A missing or unreadable TRX errors the run even if the CLI exited zero. The temporary TRX folder is removed afterward. Sources: `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/_testkit.ts:175-189`.

## Settings backback must honor

| Key | Default and meaning | MbTx value | Sources |
|---|---|---|---|
| `apexTestkit.command` | `dotnet apex`; CLI prefix | `dotnet apex` | `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/package.json:26-29`; `MbTx/.vscode/settings.json:25-28` |
| `apexTestkit.solution` | Empty; optional solution file limiting projects | Unset | `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/package.json:31-34`; `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/_testkit.ts:40-41` |
| `apexTestkit.runSettingsPath` | Empty; pass `--settings` on runs/debug | `MbAuto/RgvMbDesktop/mbingdt_apextest.runsettings` | `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/package.json:36-39`; `MbTx/.vscode/settings.json:30`; `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/_testkit.ts:167-171` |
| `apexTestkit.buildBeforeRun` | `true`; pre-run `dotnet build` | `false` | `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/package.json:41-44`; `MbTx/.vscode/settings.json:31`; `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/_testkit.ts:162-166` |
| `apexTestkit.configuration` | `Debug`; pass `-c` to discovery, run, debug, and pre-run build | `Debug` | `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/package.json:46-49`; `MbTx/.vscode/settings.json:29`; `Apex@1a0e7b2:ApexAdapters/Vscode/ApexTestkit/src/_testkit.ts:77-78,155-171` |

**BBEOD04 gate:** test the installed `0.8.2-pre` tool and the selected Testkit revision together. The current Apex source branch's `Program.cs` does not dispatch list/run modes, while the historical matching CLI does. Sources: `MbTx/.config/dotnet-tools.json:5-10`, `Apex/ApexToolhub/Apex.Cli/Program.cs:16-22`, `Apex@1a0e7b2:ApexToolhub/Apex.Cli/Program.cs:15-22`.
