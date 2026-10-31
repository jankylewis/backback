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
