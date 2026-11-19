# BB-UIX-022 — fix

id: BB-UIX-022
family: UIX
status: FIXED
source: branch `fix/readback-suspects`

## The fix that holds

The tab mark comes from the file (`Lang::of_source`, `Lang::mark`); the menu bar follows the front window: `menus(csharp)`, set on open and on each activation (`follow_languages`), the start page without C# menus.

## The wrong fix

Greying the C# menu's items only: the menu title would still say C# in a window without it.
