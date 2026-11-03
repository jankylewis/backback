# BB-PRF-004 — fix

id: BB-PRF-004
family: PRF
status: FIXED
source: branch `bb-bug-021-move-copy-lines`

## The fix that holds

One cap, `RENDER_CAP` = 10 000 chars, applied where a line is shaped
(`Columns::shape`) and where its runs are built (`styled_line`): every
path that draws or measures a line gets the bound at once.

## The wrong fix

Caching the shaped line: a cursor move hits the cache, but each typed
char on the line misses it and pays the full cost again.

## Still open

The run build is chars x spans within the cap; a span cursor would make
it linear. Not needed at 10 000 chars (2 s readback with the cap).
