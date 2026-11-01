# Toolgap index

| Id | Gap | Status | For | Workaround | Path |
|---|---|---|---|---|---|
| BB-TG-001 | netcoredbg has no `runInTerminal`; real exit code unreadable | OBSERVED | none | Label exit codes "adapter-reported, unverified" | `BB_TG_001_NETCOREDBG_NO_RUN_IN_TERMINAL.md` |
| BB-TG-002 | No one-command gate; a background shell missed `cargo` | OBSERVED | none | Export `$HOME/.cargo/bin` on PATH; check each step's exit code | `BB_TG_002_NO_ONE_COMMAND_GATE.md` |
