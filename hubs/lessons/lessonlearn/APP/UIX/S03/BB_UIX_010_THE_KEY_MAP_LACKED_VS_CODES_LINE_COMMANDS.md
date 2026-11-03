# BB-UIX-010 — lesson

id: BB-UIX-010
family: UIX
status: OPEN
source: branch `bb-bug-020-line-commands`

## Who believed what

The Cmd table was swept against VS Code's keymap (BB-UIX-008: Select All,
the comment toggle), so the common keys were in.

## What changed the belief

Sweep 4 took the line commands, another page of the same keymap: nine
keys, none mapped.

## Rule left behind

Sweep the reference keymap page by page: basic editing, line commands,
selection, navigation. One page swept says nothing about the next. A
window test does not install the app's key bindings: before mapping a
key, grep `KeyBinding::new` for it, and read the key back on the app.
