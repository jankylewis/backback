# BB-GAT-014 — lesson

id: BB-GAT-014
family: GAT
status: FIXED
source: BBEOD29

## Who believed what

A real server can be attached to a test window like in the app.

## What changed the belief

gpui's test scheduler panics on a wake from another thread.

## Rule left behind

Real tools in a window test talk to the window through the test thread
(poll and forward), never through a foreign waker.
