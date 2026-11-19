# BB-GAT-005 — A test waited for a search it had not started

id: BB-GAT-005
family: GAT
status: FIXED
date: 2026-11-11
signature: panicked at tests_panel/actions/scan/tests.rs ... the search did not end
source: the TESTS filter search branch (self-detected, before the gate)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The window test for the filter's search waited for "no list running and
no search hint" before pressing Enter. Before Enter the row shows "Enter:
search every project" on purpose, so that wait could never end; it ran
out its 60 s and failed. The product was right; the test asked for a
state the step before it could not reach.
