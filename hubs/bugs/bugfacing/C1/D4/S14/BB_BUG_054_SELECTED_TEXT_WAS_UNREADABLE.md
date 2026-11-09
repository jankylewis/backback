# BB-BUG-054 — Selected text was unreadable

id: BB-BUG-054
family: UIX
status: FIXED
date: 2026-11-05
signature: selected editor text drew light syntax colours on the light accent, near invisible
source: headmate request 2026-11-05 (editor screenshot: "copied content must be readable")
cause: the selection background was `accent` (#9ccfd8): plain text on it read at 1.29:1
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

In the dark theme, select a few lines in the editor.

Expected: the selected text stays as readable as the rest (WCAG AA, 4.5:1).
Actual: light text on a light cyan band, 1.29:1.
