# BB-BUG-061 — fix

id: BB-BUG-061
family: TST
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/panels/tests_panel/mod.rs` (`uses_apex`), `bb_crates/edge/dotnet/testkit/src/backend/apex/bb_hint.rs`

## The change

`uses_apex(root, path)` asks for both: the project references Apex AND the
workspace sets an `apexTestkit.*` key, the rule `bb test` already had. An
unreadable settings file still picks Apex, so its error shows. Listing,
running and the category run all use it. A failed Apex list now ends with
a hint: the CLI must list with `test --list --json` (Apex.Cli 0.8.2-pre or
later); else set `apexTestkit.command` or remove the keys.

## Lesson

Two front ends chose the backend with two rules. The CLI's rule was
proven; the app's was never tested against a workspace without the keys.
A CLI that ignores unknown flags turns `--list` into a full run: check
what the tool supports, not what its name suggests.

## Proof

Reproduced outside the app: `dotnet apex test --project <stub> --list
--json --no-build` with the pinned tool printed the same MSB4057.
`an_apex_project_runs_through_dotnet_test_until_the_workspace_opts_in`
(red when the opt-in check is removed) and
`a_list_by_an_apex_cli_without_ide_mode_says_how_to_fix_it` (red when the
hint is removed) pass; gate green.
