# BB-BUG-061 — An Apex project listed through an Apex CLI that cannot list

id: BB-BUG-061
family: TST
status: FIXED
date: 2026-11-09
signature: dotnet apex test exited with Some(1): ... error MSB4057: The target "VSTest" does not exist in the project.
source: headmate report 2026-11-09 (screenshot: NmcTx Tests panel, _NmcDesktop expanded)
cause: the app picked the Apex CLI for every project that references Apex; `bb test` picks it only when the workspace sets `apexTestkit.*` keys. The workspace's pinned `apex` tool (0.7.0-pre-cli-proof.1) has no IDE mode: it ignores `--list --json` and runs `dotnet test` on the listing stub
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open a solution whose test project references the Apex package, with no
`.vscode/settings.json` `apexTestkit.*` keys and an Apex CLI older than
IDE mode. Open TESTS and expand the project.

Expected: its specs are listed, as `bb test` lists them (`dotnet test`).
Actual: no specs; the row shows the CLI's `dotnet test` output on the
stand-in project and MSB4057. A Run on that project would also have gone
to the old CLI, which ignores `--specs` (not run: it targets a live site).
