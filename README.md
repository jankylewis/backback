# backback

A very lightweight, dark-by-default, open-source IDE for **C# only**, built for
maximum performance: a Rust shell in front of Microsoft's own Roslyn language
server and Samsung's `netcoredbg`.

**Status:** a working macOS desktop app. It opens a `.sln`/`.slnx` or a folder:
Explorer (New, Rename, Delete, Copy/Paste, Open Terminal Here), tabs with pins,
up to three editor panes, tree-sitter highlighting, Roslyn code intelligence
(completion, diagnostics, definition, references, rename, fixes), Go to File /
Symbol, Find in Files, Build, a netcoredbg debugger (breakpoints, step, locals,
Set Next Statement), a Test Explorer, and a terminal. Live state:
[goals board](docs/plan/bb_roadmap/GOALS.md); bugs: [hubs/failures/TRIAGE.md](hubs/failures/TRIAGE.md).

The [mascot](docs/release/bb_assets/bb_mascot/bb_mascot.svg) is an original
chibi scientist. Its [dark contact sheet](docs/release/bb_assets/bb_mascot/previews/CONTACT_SHEET.png)
shows all 10 sizes; the [main PNG](docs/release/bb_assets/bb_mascot/icon.png) is 1024 px.

## Why

- Visual Studio for Mac was retired on 2024-08-31; MonoDevelop has not been built
  since January 2020. There is no open-source native C# IDE left.
- VS Code is an Electron app with 20+ processes; Rider is heavy and paid.
- Roslyn (`roslyn-language-server`, MIT, on NuGet) and `netcoredbg` (MIT) now make
  a thin, fast, fully open-source shell possible.

## Design in one picture

```
 bb / backback (Rust)                              .NET processes (owned by the SDK)
 ┌─────────────────────────────────┐   LSP/stdio   ┌─────────────────────────────┐
 │ bb-ui   theme + frontend trait  │ ◄───────────► │ roslyn-language-server (MIT)│
 │ bb-text rope buffer, positions  │   DAP/stdio   ├─────────────────────────────┤
 │ bb-syntax tree-sitter C#        │ ◄───────────► │ netcoredbg (MIT)            │
 │ bb-dotnet SDK, .sln/.slnx, test │   argv+stdout ├─────────────────────────────┤
 │ bb-lsp / bb-dap / bb-framing    │ ◄───────────► │ dotnet build / run / test   │
 └─────────────────────────────────┘               └─────────────────────────────┘
```

Roslyn is never rewritten. The shell is fast because it does little: text, UI,
search, process plumbing. Semantics stay in .NET where they already exist.

## Layout

```
backback/
  AGENTS.md, CLAUDE.md  agent laws (outside the 5-entry count)
  bb_crates/            Cargo workspace: one lint gate, .deny.toml, Cargo.lock
    bb_core/            bb-text (rope, CharIdx/LspPoint), bb-process (child
                        groups), bb-syntax (tree-sitter C#), bb-explorer, bb-search
    edge/               bb-lsp + bb-framing, bb-dap, bb-dotnet (dotnet/bb_sdk),
                        bb-testkit (dotnet/testkit), bb-intel (Roslyn features)
    shell/              bb (bb_cli), backback (desktop), bb-ui (theme),
                        bb-audit (conventions), bb-kmap (code map)
  docs/                 design, conventions, plan, release, R&D
  hubs/                 every bug, failure, lesson and tool gap
```

Conventions (enforced by `bb-audit`, rules in [AGENTS.md](AGENTS.md)): at most
5 visible entries per folder; `.md` ≤ 100 lines, `.rs`/`.cs` ≤ 250; one backbone
marker per folder (`_` in C# trees, `bb_` in Rust and neutral trees).

## Build, gate and CLI

```sh
# toolchain (once): https://rustup.rs  then  cargo install cargo-deny
dotnet tool install --global roslyn-language-server --prerelease
bash docs/release/scripts/gate.sh   # fmt, clippy, test, deny, audit
cd bb_crates
cargo run -p bb -- doctor           # dotnet, Roslyn, netcoredbg
cargo run -p bb -- map              # code map: crates; then map <path>, --find
cargo run -p bb -- scan shell/desktop/fixtures/_Harness/Harness.slnx
cargo run -p bb -- open path/to/Solution.slnx
cargo run -p bb -- audit ..
```

`bb` also has `outline`, `sweep`, `bench`, `debug`, `probe` and `test`; run
`bb` with no command for the full usage. Agents read `bb map` before source.
`netcoredbg` is needed for F5, `bb debug` and the debugger tests: install it in
`~/.local/share/backback/netcoredbg/`, on `PATH`, or set `BACKBACK_NETCOREDBG`.

## macOS app bundle

Install from source (macOS 13+, Rust, .NET SDK): clone, then on `master` run
`bash docs/release/scripts/install.sh`. It builds `.target/bundle/backback.app`
(ad-hoc signature, via `bb_bundle.sh`) and installs `/Applications/backback.app`;
the `bb` CLI ships at `backback.app/Contents/Resources/bb`. A local build opens
with no Gatekeeper prompt. No signed download is published: backback is
distributed as source, so no paid Apple Developer ID is needed.
See [security](docs/release/community/SECURITY.md), [conduct](docs/release/community/CODE_OF_CONDUCT.md), and
[contributing](docs/release/community/bb_CONTRIBUTING.md).

## Licence

MIT. Dependencies are MIT/Apache-2.0/BSD; `cargo deny` enforces the allow-list.
`vsdbg` (Microsoft's debugger) is deliberately not used: its licence limits it to
Visual Studio and VS Code.
