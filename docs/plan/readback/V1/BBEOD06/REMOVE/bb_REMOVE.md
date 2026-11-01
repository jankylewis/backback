# BBEOD06-REMOVE readback, 2026-11-01 (backback_1)

Headmate ruling: "we dont need any MCP server ability here".

## Removed

- Crate `bb-mcp` (`edge/dotnet/mcp`) and its workspace entry.
- `bb mcp list|tools|call|log` (`cli/mcp.rs`, `bb_commands/dotnet/mcp.rs`).
- Desktop Harness panel (`panels/bb_harness`), its View menu item and
  signal/quit hooks.
- Fixture project `_Harness/McpServer`; `Capability::Mcp` in `bb-dotnet`.
- `bb_group` is not lost: BBEOD08-REAP moves it to `bb_core/process`.

## Guards (family: a removed feature still reachable)

- `no_mcp_entry_point_in_the_command_list`: parser and usage text.
- `bb_has_no_mcp_entry_point`: real `bb` binary, usage and scan output.
- `no_menu_item_opens_an_mcp_panel`: desktop menus.
- Found by the guard: an unknown word with arguments reported
  "unexpected argument". The parser now names the unknown command first.
- Fixture count tests updated for 3 projects (Roslyn load, rename, refs).

## Gate (`CARGO_BUILD_JOBS=2`, stock `BACKBACK_NETCOREDBG`)

- fmt, all-feature Clippy `-D warnings`: passed.
- `cargo test --workspace --all-features -- --test-threads=2`:
  270 passed, 0 failed, 0 ignored, 41 suites.
- `cargo deny check`: exit 0. `bb audit ..`: clean.
- Tracked absolute-path guard: 0 hits. `git diff --check`: clean.

## Served readback

- Debug `backback` on a scratch copy of `_Harness`. `NO_MCP.png` is a
  `screencapture -l` window capture: Explorer, editor, status "Ready";
  no right-side panel, no "Select MCP server".
- SIGTERM to the app: app and Roslyn children gone (0 left).
