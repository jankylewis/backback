# BB-BUG-058 — The terminal could not scroll back

id: BB-BUG-058
family: UIX
status: FIXED
date: 2026-11-06
signature: wheel or trackpad over the terminal does nothing; earlier output is out of reach
source: headmate report 2026-11-06 (screenshot: terminal, "cannot scroll up")
cause: the panel never handled the scroll wheel; the 2,000 history lines were kept but unreachable
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open the terminal, run a command with long output (`seq 1 300`), scroll
up with the wheel or two fingers.

Expected: the output scrolls back, as in VS Code.
Actual: nothing moves; only the last screen of output can be seen.
