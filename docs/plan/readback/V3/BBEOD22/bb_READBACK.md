# BBEOD22 readback: Go to Implementation and Go to Type Definition (2026-11-03)

Branch build of `bbeod-22-goto-impl-type`, `--no-write`, a fresh scratch
copy of the harness fixture per case (not restored: the reference from
`_NUnitApex` to `Core` still resolves). `state` lines are 1-based.
GreeterSpec.cs line 14 is `string actual = greeter.Hello(name);`; in
Core/Greeter.cs the class is on line 3 and `Hello` on line 5.

| Case | Keys | Result |
|---|---|---|
| T1 type definition of `greeter` | `goto:14:25 cmd-ctrl-b` | PASS: Greeter.cs, line 3, col 21 (the class name) |
| T2 implementation of `Hello` | `goto:14:33 cmd-alt-b` | PASS: Greeter.cs, line 5, col 19 (the method name) |
| T3 plain Ctrl+B | `goto:14:25 ctrl-b` | PASS: GreeterSpec.cs, line 14, col 24 (one char left) |

`problems=12` in T1 and T2 are the unrestored harness's own errors (NUnit
and xUnit types unresolved), not this change. T3 asks no question, so its
`problems=0` reads a tab the server never opened (BB-TG-015).

Several implementations open the places panel ("2 implementations"). The
harness has no interface, so that path is proven by the window test
`cmd_alt_b_jumps_to_one_implementation_and_lists_several`, not here.

## Verdict

PASS. Both commands land on the right line and column through the real
server; Ctrl+B without Cmd keeps its text-key meaning.
