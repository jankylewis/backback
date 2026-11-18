# BB-GAT-011 — lesson

id: BB-GAT-011
family: GAT
status: FIXED
source: BBEOD27 full gate, 2026-11-19

## Who believed what

Blanking literals keeps the line structure because newlines are pushed.

## What changed the belief

Only unescaped newlines were pushed; an escape jumped over one.

## Rule left behind

A text transform that must keep line numbers is tested on the count of
line breaks for every literal and comment form, not on one sample.
