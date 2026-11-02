# BBEOD15 readback: Cmd + hover link, Cmd+click to definition

Date 2026-11-03, by backback_mm. Branch `bbeod-15-cmd-link`, release build,
on a scratch copy of the `_Harness` fixture (`Core/Greeter.cs`), Roslyn
ready. Input: the script's real mouse steps; `cmdhover:` and `cmdclick:`
hold Cmd. Reference behaviour: VS Code `goToDefinitionWithMouse` (decorate
only when the definition provider returns a location).

## Run 1 (01:03, screen unlocked)

| Step | Window state | Expected |
|---|---|---|
| Cmd + hover `IsNullOrWhiteSpace` (7:33) | link chars 126-144, confirmed after 781 ms | the whole word, after Roslyn |
| hover, no Cmd | no link | cleared |
| Cmd + hover in `"there"` (7:58) | link 154-159 confirmed | **no link**: bug |
| Cmd+click `name` (7:49) | cursor 5:32, the `name` parameter | the definition |

Bug found: Roslyn answers a definition (System.String) for a word inside a
string literal, so `there` lit up. Fix: words in strings and comments never
ask and never link (the editor's own highlight). Red first:
`words_in_strings_and_comments_never_ask_and_never_link`.

## Not covered

The real-app rerun after the fix could not run: the screen was locked from
01:05 (BB-TG-010), so script mouse steps hit nothing. The fix is proven by
the window test, which sends real mouse events with Cmd through the same
handlers. The rerun is the next readback once the screen is unlocked.
