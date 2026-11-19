# BB-UIX-023 — lesson

id: BB-UIX-023
family: UIX
status: FIXED
source: BBEOD32 readback row 1a, BBEOD33 fix

## Who believed what

A cut last row looked like a clipped list.

## What changed the belief

BBEOD33 reproduced it first: a short window with more rows than fit, driven by keys.

## Rule left behind

A row inserted into a page's flow moves every heading after it; place a list's heading with the list, and every keyboard-selectable list needs a scroll handle.
