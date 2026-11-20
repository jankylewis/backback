# BB-BUG-093 — fix

id: BB-BUG-093
family: UIX
status: FIXED
source: branch `bbeod-41-python-opt-in`

## The change

`default_ticks` drops Python (a Python-only folder keeps it); `finish` never saves a found set; the `langs` store is v2 with a `picked` mark on each record and reads a v1 file as no choice; Open... pre-ticks every language but Python. With Python unticked the window also drops the PY tab mark, Build > Python Check, Python projects in Find in Files and the build icon on Python project files.

## Proof

`a_csharp_and_python_folder_with_no_saved_choice_starts_no_pyright_process` red at 7fb7aaf (`["10099 node /opt/homebrew/bin/pyright-langserver --stdio"]`), green after, with a ticked-Python control that sees the real pyright (`BACKBACK_REQUIRE_TOOLS=1`). The `unpicked` window tests and the bb-lang store and tick tests: see `evidence/BBEOD41_READY.md` in the fleet state.
