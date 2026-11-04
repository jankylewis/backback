# BB-TG-019 — Runner presses miss UI a hidden window never drew

id: BB-TG-019
status: OBSERVED
for: BB-BUG-043
date: 2026-11-04

## Gap

The readback runner sends real presses, but gpui hit-tests against the
last drawn frame. A readback window covered by other windows draws no
new frame, so a popup opened by a step has no hitbox yet: a press
"outside" it reaches nothing. Three of five presses read as failures
that a window test (which always draws) showed were not.

## Workaround

Prove mouse behaviour with window tests (`VisualTestContext` draws every
frame); use runner presses (`pressat:<x>:<y>`, new) only with the
readback window in front, and confirm a frame drew before trusting one.
