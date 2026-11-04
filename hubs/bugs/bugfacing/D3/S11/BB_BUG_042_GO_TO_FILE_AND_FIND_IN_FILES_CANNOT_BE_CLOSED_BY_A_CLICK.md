# BB-BUG-042 — Go to File / Find in Files cannot be closed by a click outside

id: BB-BUG-042
family: UIX
status: FIXED
date: 2026-11-04
signature: the Go to File / Find in Files box stays on screen after a click elsewhere
source: headmate report 2026-11-04 (screenshot: Go to File over the Explorer; "cannot close the modal")
cause: only Escape closed the box, and only while the box had key focus; a click elsewhere moved the focus, so Escape no longer reached it
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Press Cmd+Shift+O (or Cmd+Shift+F), then click the editor or the Explorer.

Expected (VS Code): the box closes. A click on a row opens that file.
Actual: the box stayed; Escape did nothing now; rows ignored clicks.
