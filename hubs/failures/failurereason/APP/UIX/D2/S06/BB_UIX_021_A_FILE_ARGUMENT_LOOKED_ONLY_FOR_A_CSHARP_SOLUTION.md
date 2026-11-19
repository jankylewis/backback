# BB-UIX-021 — A file argument looked only for a C# solution

id: BB-UIX-021
family: UIX
status: FIXED
date: 2026-11-19
signature: window title = the file's own folder; no server for a `.rs`/`.py` file
source: `app/args.rs` `split_argument` at b13fcbb
observed: BB-BUG-084
links: fix=yes lesson=yes evidence=no toolgap=none

## What it is

`split_argument` walked the file's ancestors for `find_solution` only (unbounded, to `/`); a Rust or Python file never found its `Cargo.toml` or `pyproject.toml`, so the window opened its own folder, the language detection saw one file, and no server started. The doc comment said so: the rule was written for the C#-only app and never grew with BBEOD27/28.

## How to confirm it

Open a `.rs` or `.py` file of a project with no solution; read the window title and `ps` children.
