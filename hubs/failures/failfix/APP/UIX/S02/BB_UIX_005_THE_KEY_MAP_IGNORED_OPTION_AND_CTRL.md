# BB-UIX-005 — fix

id: BB-UIX-005
family: UIX
status: FIXED
source: PRs #43 (Option), #44 (Ctrl)

## The fix that holds

The key map reads the modifiers first: with Option, Left/Right move by word
and Backspace/Delete delete by word (`Document::word_left/word_right`,
`delete_word_back/forward`, stops as VS Code and IntelliJ, `\r\n` whole);
with Ctrl, A/E/B/F/P/N/H/D map to line start, line end, char, line and
delete, Shift extending the moves; any other Ctrl key is not an edit.

## The wrong fix

Adding Option+Left alone: the sweep found four Option keys and eight Ctrl
keys in the same state.

## Proof

Red first: 6 `Document` word tests on stubs; window tests
`option_arrows_and_option_backspace_work_by_word` and
`ctrl_emacs_keys_move_and_delete` failed on the old map. Readback in the
installed app at `eadf6d0`: Ctrl+E, Right, Option+Backspace, Ctrl+A on a
CRLF file saved exactly the expected bytes.
