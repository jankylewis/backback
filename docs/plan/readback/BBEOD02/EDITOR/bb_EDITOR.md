# BBEOD02-B editor readback

1. Opened the C# fixture through the Explorer. The tab showed icon, name,
   close control, highlighted visible lines, monospace text, and line numbers.
2. Typed, undid, and closed a dirty tab with Discard after the Save/Discard/
   Cancel prompt. The dirty dot cleared after undo.
3. Opened `MbAuto/MbDesktop/Grammars/_ApexManager.cs` under MbTx and
   `src/ApexTest/Reporting/ApexReportScope.cs` under Apex read-only. Both
   rendered C# text in the live app. Neither source file was saved.
4. Compared the live editor with VS Code Insiders: both showed a C# file tab,
   line gutter, monospace text, and syntax color. VS Code also showed its
   existing breadcrumb, minimap, status bar, activity bar, and test panel.
5. On a disposable 5,000-line fixture, Cmd+S persisted exactly 500 typed `a`
   characters before the original first line. An additional unsaved `x` was
   kept off disk when window close showed Save/Discard/Cancel.

## Saved screenshots

- `BBEOD02_EDITOR_FIXTURE.png`: highlighted Greeter.cs tab and Explorer.
- `BBEOD02_EDITOR_MBTX.png`: read-only MbTx _ApexManager.cs tab.
- `BBEOD02_EDITOR_DIRTY_CLOSE.png`: unsaved fixture and close prompt.

Each PNG was captured from this seat's live window with `screencapture -l` and
visually inspected after writing. Existing BBEOD02-A Explorer captures remain
in the parent folder.

## Theme follow-up

- The native View menu exposed Toggle Light/Dark Theme and Follow macOS
  Appearance. A menu placement defect initially put both actions under the
  application menu; a failing-first desktop test and live recheck fixed it.
- With the fixture tab open, Toggle changed the running app to light. Follow
  cleared the override and returned the same window to the current macOS dark
  appearance, without restart. The appearance observer is wired for later
  system changes; a global macOS setting change was not part of this readback.
- Both live window captures were saved and inspected in `THEME/`.
