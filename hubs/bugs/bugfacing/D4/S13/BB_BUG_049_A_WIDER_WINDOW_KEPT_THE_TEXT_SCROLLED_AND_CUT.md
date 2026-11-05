# BB-BUG-049 — A wider window kept the text scrolled sideways and cut off

id: BB-BUG-049
family: UIX
status: FIXED
date: 2026-11-05
signature: after End on a long line, widening the window left the line starts cut off on the left
source: headmate report 2026-11-05 (screenshots, build 0501a5a: "when i extended the window of backback ... content on the right side still there")
cause: the sideways scroll was clamped to the widest line only on a swipe; a wider viewport never pulled it back
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open a file with a long line; press End on it; widen the window past the line's end.

Expected: the text shows from column 1, as the line now fits.
Actual: the text stayed scrolled; the first characters of every line were hidden.
