# BB-UIX-005 — lesson

id: BB-UIX-005
family: UIX
status: FIXED
source: PRs #43, #44

## Who believed what

The editor handled the arrow keys and Backspace, so moving and deleting was
done.

## What changed the belief

Using the installed app as a Mac user: Option and Ctrl combinations every
macOS text field honours did nothing, or the unmodified thing.

## Rule left behind

A key is read with its modifiers, and a new editing key is checked against
the whole modifier table (none, Shift, Option, Cmd, Ctrl, each with Shift)
before it is called done. When one key of a family is wrong, sweep the
family.
