# Upstream issue draft: Roslyn formatter fails on U+2029/U+0085 trivia

Status: DRAFT, text only. Not filed (BBEOD04-A ruling (b)).
Target: `dotnet/roslyn`, area IDE / Formatting.

## Title

`textDocument/formatting` returns `-32000 Unexpected true - file
TriviaDataFactory.CodeShapeAnalyzer.cs line 36` on a document with a
U+2029 (or U+0085) before a `//` comment

## Environment

- `roslyn-language-server` 5.13.0-1.26514.10 (dotnet global tool), stdio.
- .NET SDK 10.0.403, macOS 27.0.1, Apple M4 Pro.
- Client: backback `bb-intel` (plain LSP, no VS Code extension).

## Steps

1. Load any solution; open a `.cs` file of it with `didOpen`.
2. Text of the file (3 code points, then `//`): `"[] //"`.
3. Send `textDocument/formatting` with `{ "tabSize": 4, "insertSpaces": true }`.

## Expected

A list of text edits (possibly empty), as for any other text.

## Actual

```
server error -32000: Unexpected true - file TriviaDataFactory.CodeShapeAnalyzer.cs line 36
```

## Notes

- Minimal witness found by delta-debugging a random edit sequence from
  the backback sync property test (400 random texts, 1 failure).
- Same failure: `" // x"`, `"a // b c"`,
  `"x\u0085// y z"`.
- No failure: one U+2028, U+2029 or U+0085 alone at 54 tried spots of a
  normal file; `"// x\n"`; `" //x\n"`.
- C# treats U+0085, U+2028 and U+2029 as line breaks. The formatter's code
  shape check seems to disagree with the lexer on where the comment line ends.
- The server stays up; later requests on other documents succeed.

## What backback does meanwhile

`Session::format` turns the server error into the typed
`IntelError::Formatter { path, code, message }`. The session keeps
working. Test: `a_formatter_failure_is_a_typed_error_and_the_session_keeps_working`.
