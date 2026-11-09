---
name: cda-backback-develop
description: "Change backback code safely: worktree, failing test first, mutation check, audit before the gate, every convention kept."
metadata:
  source: skillhub
---

# cda-backback-develop

Change backback code safely: worktree, failing test first, mutation check, audit before the gate, every convention kept.

- Branch from `origin/master` in a worktree: `git worktree add ../backback_wt/<topic> -b <type>/<topic> origin/master`.
- List the bug families the change can hit and how each is blocked (type, test, audit rule). Write the failing test at the layer the behaviour lives: a unit test for pure logic, a gpui window test (`TestAppContext`, `debug_bounds` on a `debug_selector`) for anything on screen. See it fail for the right reason.
- Make the change, then a mutation check: revert the fix (copy the file to the scratchpad, edit, restore) and confirm the new test goes red; a test that stays green proves nothing.
- Conventions (enforced by `bb audit`, run it from `bb_crates/` with `cargo run -q -p bb -- audit ..` BEFORE the gate): <= 5 entries per folder with one `bb_` marker; `.rs` <= 250 lines, `.md` <= 100, functions <= 50, params <= 5; every `let` and closure parameter typed (`&mut dyn FnMut` for closures held in a let); no `unwrap`/`expect`/`panic`/`as`/`[i]` outside tests; no `_ =>` on own enums.
- Clippy pedantic traps: `similar_names` (rename), `struct_excessive_bools` (> 3 bools: use an enum or tuple). A temporary passed to `debug_bounds(&format!())` needs `.leak()`.
- A menu, key binding or action change: run the `launch::` tests before the gate; they count menu items (BB-GAT-004).
- When two front ends (the `bb` CLI and the app) make the same decision, they must share the rule; check the other one when you change either (BB-BUG-061).
- Read gpui behaviour from its source in `~/.cargo/git/checkouts/zed-*/` before assuming it (e.g. `scroll_to_item` only moves an item that is out of view; `scroll_to_item_strict` always moves).
- `cargo fmt --all`, the targeted tests, clippy on the touched crates, the audit; then hand over to cca-backback-ship.
- Load with `agentfarm skillload <seat> cda-backback-develop`.
- Codex consumes this variant: run long steps (gate, CI wait, install) as separate commands and read their exit codes; never chain a commit after a grep.
- Also follow cda-rust-dev for Rust accuracy rules.

## Guardrails

- Never read, print or copy `.env` files, auth stores or secrets; runsettings secrets are counted, never shown.
- Real solutions (MbTx, Apex, dna-nemocatalog-tx, msi, mm, swqc) are read-only: backback runs on them with `--no-write`; every writable run uses a scratch copy.
- Never reach green by `#[ignore]`, a narrowed assertion or an edited fixture.
- Never write product code in the primary checkout; never commit to `master`.
