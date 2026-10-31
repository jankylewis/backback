# backback

A very lightweight, dark-by-default, open-source IDE for **C# only**, built for
maximum performance: a Rust shell in front of Microsoft's own Roslyn language
server and Samsung's `netcoredbg`.

**Status:** R&D skeleton (M0). The crates compile into a `bb` command line that
proves the risky path: start Roslyn from Rust, load a real solution, time it.
No GUI yet; the GUI milestone (M1) is in [docs/04_ROADMAP.md](docs/04_ROADMAP.md).

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
  Cargo.toml            workspace, one lint gate for every crate
  .deny.toml            licences, advisories, duplicate crates (cargo deny reads it)
  LICENSE               MIT
  README.md
  crates/
    text/               bb-text     rope buffer, CharIdx / LineIdx / LspPoint newtypes
    syntax/             bb-syntax   tree-sitter C# parse, outline (highlighting next)
    dotnet/             bb-dotnet   SDK discovery, .sln/.slnx listing, test runner facade
    protocol/
      framing/          bb-framing  Content-Length codec shared by LSP and DAP
      lsp/              bb-lsp      JSON-RPC client + Roslyn specifics (solution/open)
      dap/              bb-dap      DAP envelope + netcoredbg discovery
    shell/
      ui/               bb-ui       theme tokens (dark default) + Frontend trait
      bb/               bb          CLI: doctor, open, outline
  docs/
    01_RND_REPORT.md    what the world does: Zed, Helix, Lapce, VS Code, Rider, Roslyn
    02_ARCHITECTURE.md  process model, crates, data flow, latency budget
    03_DECISIONS.md     ADRs: Rust, GPUI, ropey, Roslyn LSP, netcoredbg, MIT
    04_ROADMAP.md       M0..M4, estimates, risks, performance gates
    05_PROOF_M0.md      how to run the proof and what to record
```

Every folder holds at most five entries; every `.rs` stays under 500 lines and
every `.md` under 200 (the HD taxonomy used across this workspace).

## Build and gate

```sh
# toolchain (once): https://rustup.rs  then  cargo install cargo-deny
dotnet tool install --global roslyn-language-server --prerelease
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings \
  && cargo test --all-targets && cargo deny check

cargo run -p bb -- doctor
cargo run -p bb -- open path/to/Solution.slnx
cargo run -p bb -- outline path/to/File.cs
```

## Licence

MIT. Dependencies are MIT/Apache-2.0/BSD; `cargo deny` enforces the allow-list.
`vsdbg` (Microsoft's debugger) is deliberately not used: its licence limits it to
Visual Studio and VS Code.
