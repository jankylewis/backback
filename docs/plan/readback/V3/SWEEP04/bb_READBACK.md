# Sweep 4: VS Code's line commands (2026-11-03)

Each case on a fresh `Words.cs` in a folder with no solution (no Roslyn),
`BB_KEYS`, then Cmd+S; the verdict is on the saved bytes.
`Words.cs` is `class C`, `{`, `    int alpha = beta + gamma;`,
`    void M() { }`, `}`.

| Case | Keys | Installed `38baa8b` | Branch build |
|---|---|---|---|
| K1 move the line down | `goto:3:5 alt-down` | UNCHANGED | UNCHANGED (BB-BUG-021) |
| K2 move the line up | `goto:4:5 alt-up` | UNCHANGED | UNCHANGED (BB-BUG-021) |
| K3 copy the line down | `goto:3:5 shift-alt-down` | UNCHANGED | UNCHANGED (BB-BUG-021) |
| K4 delete the line | `goto:3:5 cmd-shift-k` | UNCHANGED | PASS |
| K5 select the line | `goto:3:5 cmd-l x` | `x` typed at the cursor | `x` typed at the cursor (BB-BUG-021) |
| K6 open a line below | `goto:3:5 cmd-enter q` | `q` typed into the line | PASS |
| K7 open a line above | `goto:3:5 cmd-shift-enter q` | `q` typed into the line | PASS |
| K8 indent the line | `goto:3:5 cmd-]` | UNCHANGED (Forward) | UNCHANGED: Cmd+] is Forward (BB-BUG-021) |
| K9 outdent the line | `goto:3:5 cmd-[` | UNCHANGED (Back) | UNCHANGED: Cmd+[ is Back (BB-BUG-021) |

Bugs: BB-BUG-020 (K4, K6, K7: fixed in this branch) and BB-BUG-021 (K1
to K3, K5, and K8, K9, whose keys are Back and Forward here: open), one
cause: BB-UIX-010.
