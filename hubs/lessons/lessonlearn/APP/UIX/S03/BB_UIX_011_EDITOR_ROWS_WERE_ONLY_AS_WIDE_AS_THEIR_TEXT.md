# BB-UIX-011 — lesson

id: BB-UIX-011
family: UIX
status: FIXED
source: branch `bb-bug-022-click-past-line-end`

## Who believed what

The row's text viewport is `flex_1`, so the row spans the pane.

## What changed the belief

A click past the text did nothing, and the current line's band stopped at
the text: `flex_1` grows only inside a parent with room, and the row had
none.

## Rule left behind

An element that takes clicks spans the area a user can click. A background
that ends where its content ends is the visible sign that it does not.
