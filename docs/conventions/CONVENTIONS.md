# Conventions for contributors

`AGENTS.md` is the source of truth. This page quotes its limits for humans.

## Taxonomy and backbone

- "HD taxonomy: at most 5 visible entries per folder (dotfiles and the agent
  files `AGENTS.md`/`CLAUDE.md` excepted)."
- "Backbone marker: C# trees use `_` for the one most critical entry per
  folder; Rust and neutral trees use `bb_`. A Cargo crate's `src/` needs no
  marker."

## Size caps

- "Caps: `.md` ≤ 100 lines; `.rs`/`.cs` ≤ 250; function ≤ 50; Rust `impl`/type
  and C# class ≤ 200; parameters ≤ 5."

## Explicit types

- "Explicit types: every Rust `let` and closure parameter is typed; C# never
  uses `var`. No `unwrap`/`expect`/`panic`/`as` casts/`[i]` outside tests."
- `bb_crates/shell/desktop/fixtures/_Harness/.editorconfig` sets all three C# `csharp_style_var_*`
  rules to `false:error`, disables IDE0007, and sets IDE0008 to `error`.

## No vacuous skip

- A test whose tool is missing (netcoredbg, dotnet, Roslyn) fails with a
  clear message; it never returns early and green. `bb audit` rule
  `VacuousSkip` flags a bare `return` or `return Ok(())` under a tool probe
  (`locate(`, `which(`, `.exists()`, `Command::new(` ...) in test code.

## Gate

Run from `bb_crates/`; all four commands must pass:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

Report the observed test counts and any failure. The gate in `AGENTS.md`
controls if this page or an older research note differs.
