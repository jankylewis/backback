# BBEOD02-A readback

- Opened the `_Harness` fixture, MbTx, and Apex in the desktop window.
- MbTx and Apex were read only. Screenshots: `BBEOD02_FIXTURE.png`,
  `BBEOD02_MBTX.png`, and `BBEOD02_APEX.png` in this folder.
- Compared the visible window with the requested Explorer list: dark palette,
  chevrons, nested folders, C# and Markdown labels, dimmed `obj`/`bin`/`.git`,
  and a selected row outline. The synthetic fixture showed these in live use.
- Mouse click expanded Core. Right on an expanded folder selected its first
  child; Up, Down, Left, Right, and Enter have handlers and a focused window.
- A path tooltip is bound to every row. It did not appear in the captured
  hover state, so its live appearance remains unverified.

## Issues and lessons

- Opening `/` showed an empty root label. A failing model test required `/`;
  the name fallback now displays it.
- A nested `.slnx` showed an expansion chevron but could not expand. A failing
  model test now requires it to be a leaf; the live tree no longer shows the
  false chevron.
- Right on an expanded folder collapsed it. A failing desktop navigation test
  required selection to move to the first child; the handler now does that.
- macOS may report a temporary folder via `/var` or `/private/var`. A failing
  model test exposed path lookup failure; root and requested paths now resolve
  to the same canonical identity.

## Reopened visual review

The refreshed `BBEOD02_MBTX.png` shows MbTx expanded through
`MbAuto/RgvMbDesktop/Tests/Documents/Pdf`. The refreshed
`BBEOD02_FIXTURE.png` shows every file icon kind in a synthetic folder.
Compared against the headmate screenshot criteria in the BBEOD02 brief:

| Requested item | Observed in live window |
| --- | --- |
| Closed and open folder icons | Distinct authored SVG outlines in both screenshots. |
| C# file glyph | Authored file/C# SVG beside `Foo.cs` in fixture. |
| Markdown glyph | Authored M/down-arrow SVG beside `README.md` in fixture. |
| Project and solution icons | Distinct authored SVGs beside `App.csproj` and `App.slnx`. |
| Generic file icon | Authored SVG beside `notes.txt`. |
| Vertical indent guides | One line per depth in the MbTx chain through Pdf. |
| Hide `.git` and `.DS_Store` | Absent from fixture; failing model test added first. |
| Dim ignored entries and `bin`/`obj` | `scratch.tmp`, `bin`, and `obj` dim in fixture; ignored MbTx folders dim. |
| Selected row outline | Visible on Pdf in MbTx screenshot. |

The exact headmate image is unavailable in this pane, so this compares its
stated criteria. A path tooltip remains bound to rows; its live appearance
was not visible in the captured hover state.

## Review round 2

`BBEOD02_MBTX.png` was recaptured with
`MbAuto/RgvMbDesktop/Tests/Documents/Pdf` expanded. `Pdf` is at depth 6;
`_HvFvFromTableFvStandalone` and its four sibling folders are at depth 7.
The child chevron, icon, and label begin one guide cell (16 px) to the right
of `Pdf`; the guides stay vertical instead of stacking. A row layout test was
red before the fix and now checks fixed guide offsets for depths 0 through 12.

| Headmate review item | Live readback |
| --- | --- |
| Pdf descendants | `_HvFvFromTableFvStandalone`, `FromTableFv`, `Fv`, `Hv`, and `Standalone` all appear one level below `Pdf`. |
| Long name at 300 px | The first child ends with an ellipsis at the panel edge; no partial glyph is visible. |
| Drag edge | `BBEOD02_RESIZED.png` in `docs/bb_design/` shows the same rows after dragging the edge from 300 to 500 px; the long name fits. |
| Project icon | `_RgvMbDesktop.csproj` has a distinct authored project SVG. |
| Icon colours | `AssemblyInfo.cs` is blue; folders are neutral; the project icon is violet. Markdown uses a blue-grey token (fixture capture). |
| Ignored rows | `bin`, `obj`, and an ignored runsettings file remain dim. |
