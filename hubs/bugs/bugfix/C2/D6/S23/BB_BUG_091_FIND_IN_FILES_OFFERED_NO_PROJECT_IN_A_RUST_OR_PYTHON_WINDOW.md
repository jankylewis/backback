# BB-BUG-091 — fix

id: BB-BUG-091
family: UIX
status: FIXED
source: branch `fix/csharp-assumption-sweep`

## The change

`Lang::of_project` (a `.csproj`, `Cargo.toml`, `pyproject.toml` or `setup.py`; never a solution or `requirements*.txt`) used by `project_of` and the project walk.

## Proof

`find_in_files_offers_each_cargo_crate_as_a_project` red at 88177da (`["Workspace · ws", "Folder · alpha/src"]`), green after; M2 (C# only) red. `a_project_is_a_csproj_cargo_toml_pyproject_or_setup_py_never_a_solution` (bb-lang); M3 (solutions count) red.
