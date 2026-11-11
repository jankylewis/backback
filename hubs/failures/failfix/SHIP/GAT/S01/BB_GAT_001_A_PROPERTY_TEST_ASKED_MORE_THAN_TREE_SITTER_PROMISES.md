# BB-GAT-001 — fix

id: BB-GAT-001
family: GAT
status: FIXED
source: branch `bb-syn-incremental`

## The fix that holds

The tests assert tree-sitter's contract: the text extent always matches a
fresh parse; the trees match exactly whenever the fresh parse has no error;
the CI case is pinned and, after undoing back to the valid source, the
trees match exactly again. Tests moved to `highlight_tests.rs` (file cap).

## Proof

The pinned CI case passes; the property test passes 3,000 random edit
sequences (`PROPTEST_CASES=3000`, 91 s) with no divergence on valid text.
