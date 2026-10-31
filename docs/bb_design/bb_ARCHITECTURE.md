# 02 · Architecture

## Process model

```
            ┌──────────────── backback (one Rust process) ────────────────┐
 keyboard → │ frontend (GPUI, M1)                                         │
 mouse    → │   └─ views: editor · explorer · problems · tests · debug    │
            │ bb-ui: theme tokens, Frontend trait                          │
            │ editor core (sync, no await):                                │
            │   bb-text  rope buffer, selections, undo                     │
            │   bb-syntax tree-sitter C#: highlight, fold, outline        │
            │ workspace (M1): open docs, solution tree, settings           │
            │ process edge (async, every call has a deadline):             │
            │   bb-lsp ──stdio──► roslyn-language-server   (semantics)     │
            │   bb-dap ──stdio──► netcoredbg               (debugging)     │
            │   bb-dotnet ─argv─► dotnet build/run/test     (SDK)          │
            └──────────────────────────────────────────────────────────────┘
```

Three external processes, all MIT, all owned by the .NET SDK or a `dotnet tool`.
backback never links .NET; it speaks three text protocols over pipes.

## Crate graph (no cycles; arrows = depends on)

```
bb ──► bb-ui
   ──► bb-dotnet
   ──► bb-lsp ──► bb-framing
   ──► bb-dap ──► bb-framing
   ──► bb-syntax
(bb-text is used by the editor core in M1; bb depends on it then)
```

| Crate | Owns | Must never |
|---|---|---|
| `bb-text` | rope buffer, `CharIdx`/`LineIdx`/`LspPoint` newtypes, checked conversions | know about LSP, files or UI |
| `bb-syntax` | tree-sitter C# parse, outline (highlight queries next) | call Roslyn |
| `bb-dotnet` | SDK discovery, `.sln`/`.slnx` listing, `dotnet` runner with deadlines, test platform enum | evaluate MSBuild itself |
| `bb-framing` | `Content-Length` codec (incremental, skips bad frames) | know message meaning |
| `bb-lsp` | JSON-RPC client (writer task, reader task, id map, deadlines) + Roslyn specifics | block the UI thread |
| `bb-dap` | DAP envelope types, netcoredbg discovery (session machine in M4) | use `vsdbg` |
| `bb-ui` | theme tokens (dark default), `Frontend` trait, `Headless` | depend on a GUI crate |
| `bb` | CLI: `doctor`, `scan`, `open`, `outline`, `audit`, `bench` (M0 proof) | contain editor logic |

## Data flow for one keystroke (M1 target)

1. Frontend receives the key → `editor.insert(CharIdx, "x")` on the main thread
   (`bb-text`, O(log n), no allocation beyond the rope node).
2. `bb-syntax` re-parses incrementally with the old tree (µs). Highlight spans
   for the visible range only.
3. Frame renders from the display map. **Budget so far: ≤ 2 ms.**
4. After the frame: `bb-lsp` sends `textDocument/didChange` (debounced ~50 ms).
   Diagnostics arrive later as notifications and paint on the next frame.

Nothing in steps 1–3 awaits. Roslyn can be slow without the editor feeling slow.

## Positions: one rule
UI and rope use `CharIdx`. LSP uses `LspPoint` (line, UTF-16 column). The only
bridge is `Buffer::to_lsp` / `Buffer::from_lsp`, both checked. A surrogate pair
(`😀`) is 1 char and 2 UTF-16 units; the tests prove the mapping.

## Concurrency
- Editor core: single-threaded, synchronous. State lives in the frontend's
  entity model (GPUI) or plain structs (headless).
- Process edge: tokio. One writer task and one reader task per child process.
  Requests resolve through oneshot channels; server notifications arrive on an
  mpsc channel the UI drains once per frame.
- Locks: `std::sync::Mutex` only around the pending-request map, never across an
  await. Outgoing bytes go through a channel, so no lock guards the pipe.
- Every request has a deadline (`LspError::Timeout`), every process call too
  (`SdkError::Timeout`). A dead server fails all waiters with `Closed`.

## Failure model
- Typed error enum per crate (`thiserror`); no sentinel values, no panics outside
  tests (lints deny `unwrap`/`expect`/`panic`/`indexing_slicing`/`as`).
- A bad frame is dropped and the connection continues (`bb-framing` test).
- Roslyn crash → the UI shows "language server stopped" and offers restart; the
  editor keeps working with tree-sitter highlighting (local, never waits).

## Performance gates (measured, not promised)
| Gate | Target | How |
|---|---|---|
| Cold start → editable | ≤ 300 ms | `hyperfine` on the binary, empty window |
| Keypress → pixel | p50 ≤ 8 ms, p99 ≤ 16 ms | Typometer on a 5k-line file |
| Idle RSS, no Roslyn | ≤ 150 MB | `ps`/Activity Monitor after 60 s |
| Solution load, MbTx (2,080 .cs) | report, don't cap (Roslyn-bound) | `bb open` prints it |
| Outline parse, 50 kB file | ≤ 5 ms | `bb outline` prints it |

## Settings and files
- `~/.config/backback/settings.toml` (TOML, `deny_unknown_fields`, versioned).
- Per-solution state (open files, breakpoints) in `.backback/` next to the
  solution, git-ignored by a generated entry.
- Logs: Roslyn's own under the OS temp dir (`backback-roslyn-logs`); ours via
  `tracing` (M1) to stderr and a rolling file. No telemetry, no network calls
  except what `dotnet`/NuGet do themselves.
