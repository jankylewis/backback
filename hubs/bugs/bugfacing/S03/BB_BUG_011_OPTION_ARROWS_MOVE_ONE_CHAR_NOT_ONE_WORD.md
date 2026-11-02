# BB-BUG-011 — Option+arrows move one char, not one word

id: BB-BUG-011
family: UIX
status: FIXED
date: 2026-11-03
signature: Option+Backspace deletes a single char
source: BB-BUG-010 family sweep in `/Applications/backback.app` at `24aa57c`
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Open any file; put the cursor at the start of `class A`; press
Option+Right three times; type `x`. Or press Option+Backspace after a word.

Expected (every macOS editor; VS Code, IntelliJ): Option+Left/Right move
by word, Option+Backspace/Delete remove a word.
Actual: the editor ignored Option: three Option+Right moved three chars
(`claxss A`), Option+Backspace removed one char.

Goal: BBEOD02 (editor).
