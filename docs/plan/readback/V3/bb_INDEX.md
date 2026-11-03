# Readback V3: units from 2026-11-03 evening on

`V2/` holds five entries, its cap; new units go here. One folder per unit,
each with the steps, captures and verdict of a read back on the real app.

| Unit | What was read back |
|---|---|
| `BBEOD22/` | Go to Type Definition (Cmd+Ctrl+B) and Go to Implementation (Cmd+Alt+B) land on the harness's `Greeter`; plain Ctrl+B still moves left |
| `TG015/` | BB-TG-015: the readback runner flushes after each step, so a run that asks no question reads the server's own list (`version=Some(1)`, was `None`) |
