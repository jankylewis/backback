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

CUA screenshots were observed during these steps, including the window close
prompt. The CUA surface did not provide a screenshot file-write operation, so
editor screenshots are not persisted in this folder. Existing BBEOD02-A
Explorer captures remain here. The editor screenshot artifact is still open.
