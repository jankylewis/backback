# BB-BUG-013 — Long lines are clipped at the pane's edge; no horizontal scroll

id: BB-BUG-013
family: UIX
status: FIXED
date: 2026-11-03
signature: cannot horizontally scroll
source: headmate screenshot 2026-11-03 ~15:30: the installed app on `PipelineConfigContractTests.cs` (MbTx), lines wider than the pane
observed: "cannot horizontally scroll, make the right-side content invisible, cannot view them"; every long line cut at the pane's right edge, a trackpad swipe did nothing, End put the caret out of sight
cause: BB-UIX-006
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open a file with lines wider than the editor pane (a long test method name,
an assertion message); swipe sideways on the trackpad, or press End on
such a line.

Expected: the text scrolls so the rest of the line can be read; the caret
stays in sight.
Actual: nothing scrolls; the line ends under the pane's edge and the caret
goes with it.

Goal: BBEOD02 (editor). Critical: code past the pane is unreadable.
