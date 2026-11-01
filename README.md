# backback

A very lightweight, dark-by-default, open-source IDE for **C# only**, built for
maximum performance: a Rust shell in front of Microsoft's own Roslyn language
server and Samsung's `netcoredbg`.

**Status:** early macOS desktop build. The `backback` app opens a C# solution.
The `bb` command line provides doctor, scan, open, outline, and audit tools.
See [the roadmap](docs/plan/bb_roadmap/bb_ROADMAP.md) for planned work.

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
  LICENSE, README.md
  bb_crates/            Cargo workspace: one lint gate, .deny.toml, Cargo.lock
    bb_core/
      bb_text/          bb-text     rope buffer, CharIdx / LineIdx / LspPoint newtypes
      syntax/           bb-syntax   tree-sitter C# parse, outline
    edge/
      bb_lsp/           bb-lsp      JSON-RPC client + Roslyn specifics
      framing/          bb-framing  Content-Length codec shared by LSP and DAP
      dap/              bb-dap      DAP envelope + netcoredbg discovery
      dotnet/           bb-dotnet   SDK, .sln/.slnx, projects, runsettings, tests
    shell/
      bb_cli/           bb          CLI: doctor, scan, open, outline, audit
      desktop/          backback    native macOS GUI
      ui/               bb-ui       theme tokens (dark default)
      audit/            bb-audit    convention audit; `cargo test` runs it on the repo
  docs/                 design, plan, R&D
  bb_crates/shell/desktop/fixtures/_Harness/    small C# solution for readback
```

Conventions (enforced by `bb-audit`, rules in [AGENTS.md](AGENTS.md)): at most
5 visible entries per folder; `.md` ≤ 100 lines, `.rs`/`.cs` ≤ 250; one backbone
marker per folder (`_` in C# trees, `bb_` in Rust and neutral trees).

## Build and gate

```sh
# toolchain (once): https://rustup.rs  then  cargo install cargo-deny
dotnet tool install --global roslyn-language-server --prerelease
cd bb_crates
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings \
  && cargo test --workspace && cargo deny check
cargo run -p bb -- doctor
cargo run -p bb -- scan shell/desktop/fixtures/_Harness/Harness.slnx
cargo run -p bb -- open path/to/Solution.slnx
cargo run -p bb -- outline path/to/File.cs
cargo run -p bb -- audit ..
```

`netcoredbg` is optional for editing; `bb debug` and debugger tests need it.
Set `BACKBACK_NETCOREDBG=/path/to/netcoredbg` or put it on `PATH`.

## macOS app bundle

`bash docs/release/scripts/bb_bundle.sh` builds `.target/bundle/backback.app`
(Cargo version, ad-hoc signature). `install.sh` (same folder) builds it from a clean `master`
and installs `/Applications/backback.app`. The `bb` CLI ships at
`backback.app/Contents/Resources/bb`; macOS 13+.
For public distribution, the maintainer must use a Developer ID and Apple
notarization via [`sign_release.sh`](docs/release/scripts/sign_release.sh).
See [security](docs/release/community/SECURITY.md), [conduct](docs/release/community/CODE_OF_CONDUCT.md), and
[contributing](docs/release/community/bb_CONTRIBUTING.md).

## Licence

MIT. Dependencies are MIT/Apache-2.0/BSD; `cargo deny` enforces the allow-list.
`vsdbg` (Microsoft's debugger) is deliberately not used: its licence limits it to
Visual Studio and VS Code.
