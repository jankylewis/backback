# BB-PRF-007 — lesson

id: BB-PRF-007
family: PRF
status: FIXED
source: branch `perf/lang-no-regression` (BBEOD36)

## Who believed what

The language walk had a 400 ms deadline and stopped early once every
language was seen, so it looked bounded and cheap. Its tests used the
seven small fixture folders, and every saved-choice launch cost nothing.

## What changed the belief

An A/B of release bundles from before and after the picker, launched the
way a user launches them, on a real solution and on a C#-only tree at
the walk's scale: a C#-only folder never sees Rust or Python, so it is
exactly the case that reads the whole tree, and that time sat on the UI
thread before the first frame (+74 % on MbTx, +49 % on 50,000 files).

## Rule left behind

A deadline bounds the worst case; it does not make the cost free. Any
walk that grows with the folder runs off the UI thread (BB-PRF-002 said
the same for Go to File), and a new feature on the launch path gets an
A/B of first frame against the commit before it, on a folder of C# only.
