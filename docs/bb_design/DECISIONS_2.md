# 03 · Decisions (ADRs), continued

`DECISIONS.md` holds ADR-1 to ADR-10; same format.

## ADR-11 · Language picker: C#, Rust, Python (BBEOD26, headmate)
**Context.** The headmate wants Rust and Python projects in backback without
paying for them in a C# window, and without a second debugger.
**Decision.**
- The start page ticks C#, Rust, Python; the choice is saved per workspace
  (`langs`, next to `recent`, versioned). Open is disabled with none ticked.
- `active = ticked ∩ detected` (crate `bb-lang`) is the one rule every tool
  start asks: Roslyn at open, netcoredbg and test discovery on a click. C#
  tools start only with C# active; a C#-only window starts Roslyn and
  nothing else, as before.
- Debug is C# only, ever: without C# active the Debug keys, menu items and
  breakpoints are absent; in a mixed window breakpoints take C# files only.
- Rust and Python files of an active language get tree-sitter colours
  (`tree-sitter-rust`, `tree-sitter-python`, MIT); a file of an inactive
  language opens as plain text. No Rust or Python server yet (BBEOD27/28).
- File > Languages... reopens the window with the new ticks; servers never
  start or stop live.
**Seat's calls (headmate may overrule).**
- A folder holding no language starts with C# ticked (an empty or notes
  folder still opens; a status note says "No C# project here").
- A file of no language (`.json`, `.md`) keeps the C# colours it always had
  while C# is active, else plain text.
- Detection walks at most 400 ms (dot, git-ignored and build folders
  skipped); with ticks known it stops once each ticked language is seen.
  Detected languages are saved only after a complete walk.
- File > Languages... refuses while a tab is unsaved; untitled tabs keep C#.
**Consequences.** ADR-10's "C# only" now means "debugging and servers C#
only"; the scope of other languages grows one EOD at a time.

## ADR-12 · Python is opt-in (BBEOD41, headmate ruling 2026-11-20)
**Context.** A C# repo holding `.py` helpers (MbTx, Apex) ticked Python by
default and started pyright with no choice made; the found set was then saved
as if the user had picked it. Ruling, verbatim: "pyright and python stuff
should only be started once it is checked at lang option picker ... if only
tick c# -> ignore everything about python in the opened repo".
**Decision.**
- `default_ticks` (no saved pick) is what the folder holds without Python;
  Rust and C# keep ADR-11's default.
- Only a user's pick is saved: Start Page Open with ticks, File > Languages...
  A found set is never saved (supersedes ADR-11's "Detected languages are
  saved only after a complete walk").
- The `langs` store is v2: each record carries the mark `picked`. A v1 file
  cannot tell picks from auto-saves, so it reads as no choice; the next pick
  writes v2. An unknown header, mark or key is refused.
- With Python unticked the window ignores Python: no pyright, pytest, Python
  build, Python Check menu item, PY tab mark, Python projects in Find in
  Files, Python build icon, Python in the status bar; `.py` opens plain.
**Seat's calls (headmate may overrule).**
- A folder holding only Python pre-ticks Python (else nothing works there).
- Open... pre-ticks every language but Python; an untouched Open... still
  takes the picked folder's default ticks.
- A tab's language mark follows its colours, so an unticked Rust or C# file
  also shows no mark.
**Consequences.** Every user loses their v1 picks once (they read as no
choice); C# picks come back as the default anyway. Folders without a pick walk
whole on each open (BB-PRF-009 keeps the window from waiting).
