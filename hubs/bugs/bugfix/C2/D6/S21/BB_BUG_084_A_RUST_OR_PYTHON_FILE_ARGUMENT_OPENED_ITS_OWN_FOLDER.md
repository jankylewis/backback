# BB-BUG-084 — fix

id: BB-BUG-084
family: UIX
status: FIXED
source: branch `fix/readback-suspects`; `split_argument` -> `bb_root::project_root`

## The change

`split_argument` asks `bb_root::project_root`: the nearest root of the file's language (Cargo workspace from `cargo metadata`, else the nearest `Cargo.toml`; `pyproject.toml`/`setup.py`; a solution for C# as before), then the outermost root of another language holding it. The walk stops at a git root and never reaches the home folder.

## Proof

`args::tests::a_rust_or_python_file_argument_roots_at_its_project` red at b13fcbb (4 fixtures: `rust_only`, `py_only`, `py_rust` x2 rooted at `src/`/`tool/`), green after; window test `a_rust_file_argument_opens_its_project_and_starts_its_servers`; six `args::root_tests` (cargo seam, git and home bounds, mixed, no project). Mutations M1, M1b-e red.
