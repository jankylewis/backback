# 03 · Decisions (ADRs)

Format: context → decision → consequences → revisit when. Status is `accepted`
unless noted.

## ADR-1 · Rust for the shell, .NET for the brain
**Context.** The IDE must be very light and fast; C# semantics already exist in
Roslyn (C#, MIT). A C# + Avalonia shell could host Roslyn in-process but ships a
runtime and GC in the UI path; Rust gives a native binary, no GC pauses, and
matches the workspace's kernel direction.
**Decision.** Rust shell; Roslyn, netcoredbg and `dotnet` as child processes
over LSP, DAP and argv. Never rewrite Roslyn.
**Consequences.** Two runtimes on the machine (fine: the SDK is required anyway).
Richer-than-LSP features (custom refactorings) go through Roslyn's LSP
extensions, not in-process APIs.
**Revisit when.** A needed feature has no LSP surface and Roslyn's extension
mechanism cannot reach it.

## ADR-2 · GPUI for the GUI, behind a trait
**Context.** Candidates: GPUI (Zed; GPU, editor-proven, pre-1.0, unofficial
crates.io), iced (stable-ish, wgpu, cosmic-text), Floem (Lapce; moved to parley
2026-03), Slint, egui (immediate mode; poor fit for large text).
**Decision.** GPUI pinned by git revision for M1, isolated in `bb-ui`'s
`Frontend` trait; core crates stay GUI-free. `gpui-component`'s editor is the
reference for a 200K-line text view.
**Pin.** `ba63a022c9a22ca66d68ef2a8f0bb9b5b0a9984d` from
`zed-industries/zed`. This fixes GPUI and `gpui_platform` to one source tree;
the API changes between upstream revisions. `.deny.toml` permits that git
source only for this pin.
**Shader build.** Enable `gpui_platform/runtime_shaders` on macOS because this
host lacks full Xcode's `metal` compiler. Metal compiles shaders at runtime;
five fresh MbTx launches after warm-up reached the first frame in
`146, 151, 152, 157, 156 ms` (median `152 ms`). This total includes runtime
shader work. The isolated shader overhead cannot be measured here without a
precompiled-shader build from full Xcode.
**Consequences.** Breaking upstream changes are absorbed in one crate; `Headless`
keeps the CLI and tests running without a window.
**Revisit when.** GPUI 1.0 ships (drop the pin) or its API churn costs more than
a week per quarter (switch to iced or Floem).

## ADR-3 · ropey, not crop
**Context.** `crop` benchmarks faster (byte-indexed); `ropey` is char-indexed,
mature, and offers UTF-16 conversions LSP needs.
**Decision.** `ropey` behind `bb-text`; positions are newtypes so a swap touches
one crate.
**Revisit when.** Profiling shows the rope in the top frame of a typing trace.

## ADR-4 · Official `roslyn-language-server`, stdio
**Context.** Options: OmniSharp (older, slower), `csharp-ls` (community, deprecated
in favour of the official server), the official Roslyn server (MIT, NuGet, runs
outside VS Code since 2025, daemon or stdio).
**Decision.** Official server over `--stdio`; locate via `BACKBACK_ROSLYN_LS`,
`PATH`, `~/.dotnet/tools`. `solution/open` after `initialized`; wait for
`workspace/projectInitializationComplete` before trusting semantics.
**Consequences.** We track a prerelease; `bb doctor` prints the version. Daemon
mode (shared server across clients) is a later option.
**Revisit when.** Microsoft publishes a stable release line or changes flags.

## ADR-5 · netcoredbg, never vsdbg
**Context.** `vsdbg` is licensed only for Visual Studio and VS Code. `netcoredbg`
(Samsung) and `dncdbg` are MIT with DAP.
**Decision.** `netcoredbg` with `--interpreter=vscode`. Feature gaps (e.g. some
evaluation cases) are documented, not hidden.
**Revisit when.** Microsoft relicenses `vsdbg` for third-party editors.

## ADR-6 · Test platform as an enum
**Context.** VSTest and Microsoft.Testing.Platform differ in flags and output;
MTP is opt-in via `global.json` on .NET 10 and forbids mixing in a solution.
**Decision.** `TestPlatform::{VsTest, Mtp}` with per-variant parsers; M3 uses MTP
JSON (`--list-tests json`) when the SDK has it.
**Revisit when.** MTP becomes the default and VSTest is removed.

## ADR-7 · Dark by default, tokens not colours
**Decision.** `Theme` is a struct of semantic tokens; `DARK` is returned for "no
preference", `LIGHT` on request or OS preference. Widgets never hold raw colours.
Palette inspired by Rosé Pine (MIT).

## ADR-8 · MIT licence, `cargo deny` allow-list
**Decision.** backback is MIT. Allowed dependency licences: MIT, Apache-2.0 (incl.
LLVM exception), BSD-2/3, ISC, Zlib, Unicode-3.0, MPL-2.0. Unknown registries and
git sources are denied except the pinned GPUI revision (added in M1 with reason).

## ADR-9 · Lint gate and taxonomy
**Decision.** Workspace lints deny `unwrap`, `expect`, `panic`, `todo`,
`unimplemented`, `indexing_slicing`, `as_conversions`,
`wildcard_enum_match_arm`, `dbg_macro`; `unsafe_code` is forbidden. One command
gates every change: `fmt --check`, `clippy -D warnings`, `test`, `deny check`.
Folders hold ≤ 5 visible entries; `.rs`/`.cs` ≤ 250 lines; `.md` ≤ 100 lines.
Functions hold ≤ 50 lines; Rust `impl`/type and C# class ≤ 200; parameters ≤ 5.
`AGENTS.md` is the source of truth for the complete convention and gate.

## ADR-10 · Scope: C# only, no marketplace
**Decision.** v1 supports C# projects and solutions. No extension marketplace, no
remote development, no non-.NET languages. Themes: dark and light only.
**Consequences.** The binary stays small and the surface stays testable. Anything
"pluggable" is a Rust crate behind a trait, compiled in.
**Revisit when.** v1 ships and real users ask for F# or Razor.
