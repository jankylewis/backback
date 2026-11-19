# BB-UIX-022 — lesson

id: BB-UIX-022
family: UIX
status: FIXED
source: BBEOD32 readback rows 2a, 3a, 4, BBEOD33 fix

## Who believed what

A `C#` mark might be a workspace badge, so it was logged as a suspect.

## What changed the belief

BBEOD33 reproduced it first: the same mark on `.py` and `.rs` tabs and a C# menu in a C#-less window.

## Rule left behind

When a window becomes language-aware, grep the old language's literal (`"C#"`) across the chrome and tie each one to the language rule, keys, buttons and menus alike.
