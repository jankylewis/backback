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
