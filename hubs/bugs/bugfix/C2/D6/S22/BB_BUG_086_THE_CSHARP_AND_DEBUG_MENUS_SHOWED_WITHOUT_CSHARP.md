# BB-BUG-086 — fix

id: BB-BUG-086
family: UIX
status: FIXED
source: branch `fix/readback-suspects`; `bb_window::menus(csharp)`, `follow_languages`

## The change

`menus(csharp)` adds Debug and C# only with C# active; `follow_languages` sets the bar when a workspace opens and each time its window comes to the front; the start page sets the bar without them.

## Proof

`the_csharp_and_debug_menus_show_only_while_csharp_is_active` red at b13fcbb (`[.., Debug, C#, ..]`), green after; mutations M3 (always C#) and M3b (no re-set on activation) red.
