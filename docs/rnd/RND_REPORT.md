# 01 · R&D report: how the world builds editors, and what that means for backback

Researched 2026-10-31 from project docs, blogs, NuGet, and GitHub. Figures are
quoted with their source; vendor and single-blog numbers are marked. Claims
were checked only against the cited page; verify before quoting elsewhere.

## 1. The gap backback fills

| Fact | Source |
|---|---|
| Visual Studio for Mac retired 2024-08-31; Microsoft points to VS Code or Windows | learn.microsoft.com/lifecycle |
| MonoDevelop archived, not built since January 2020; core LGPLv2, parts MIT X11 | github.com/mono/monodevelop |
| SharpDevelop: Windows-only, dormant | community write-ups |
| `roslyn-language-server` on NuGet, **MIT**, v5.13.0-1.26514.10 (2026-10-15), installs as a `dotnet tool` | nuget.org/packages/roslyn-language-server |
| `netcoredbg` (Samsung) MIT, implements DAP (`--interpreter=vscode`) and GDB/MI | github.com/Samsung/netcoredbg |
| `vsdbg` licence: only with Visual Studio, VS Code, VS for Mac; VS Code forks cannot ship it | ArchWiki, itwriting.com |
| `.slnx` is the `dotnet new sln` default in .NET 10; MSBuild ≥ 17.13 reads it; `Microsoft.VisualStudio.SolutionPersistence` reads both formats | neteye-blog, InfoQ 2025-04 |

Conclusion: for the first time all the heavy parts (compiler, language server,
debugger) are open source under MIT and runnable outside Microsoft's editors. The
only missing piece is a fast, small shell. That is the whole project.

## 2. What the fast editors do

### Zed (Rust, GPUI)
- Built **GPUI** because "no amount of optimization on top of a retained-mode DOM
  … will outperform rendering graphics like a video game" (Zed hiring blog).
- One data structure, the **SumTree** (B+ tree with summaries), powers the rope,
  the CRDT and the syntax map. Rope chunks ≤ 128 chars.
- Editor = `MultiBuffer` (text model) → `DisplayMap` (folds, wraps, inlays) →
  `EditorElement` (render). Clean layering we copy.
- Vendor claims: < 1 s startup, < 10 ms typing latency, ~600 MB RAM
  (zed.dev/compare/vscode). Independent blogs: 0.4–0.6 s start, 180–222 MB idle.
- GPUI is **pre-1.0**; crates.io only has `gpui-unofficial`/community forks
  (1.22.0-pre, 2026-09-24). Longbridge's `gpui-component` ships a code editor
  "stable at 200K lines" with tree-sitter and LSP diagnostics.

### Helix (Rust, terminal)
- Crate split: `helix-core` (rope, transactions, tree-sitter), `helix-view`
  (frontend-agnostic editor state), `helix-term` (TUI), `helix-lsp`,
  `helix-dap`, `helix-event`. Core "heavily based on CodeMirror 6".
- Lesson: keep the view model separate from the frontend from day one; Helix's
  docs admit `helix-view` is "currently tied to the terminal UI".

### Lapce and xi-editor (Rust)
- Lapce uses Floem (own UI, wgpu) and xi's rope ("rope science"). xi's rule:
  "the editor should never make you wait for anything". xi is discontinued; its
  async-everything design was too complex for the gain. Lesson: async at the
  process boundary (LSP, build), synchronous inside the editor core.

### VS Code and Rider (the incumbents)
- VS Code: Electron; blogs measure 23 processes / 3.5 GB with a folder open and
  extensions, 300–500 MB bare. Each extension adds 20–100 MB and 200–500 ms start.
- Rider: no public 2026 measurements found; qualitatively the heaviest option.
- Both carry a full extension marketplace. backback deliberately does not.

## 3. Latency: what "fast" means in numbers

- Pavel Fatin, *Typing with pleasure*: total latency = input + processing +
  output; editor processing should be a few ms so the keyboard and display
  dominate. His Typometer tool measures keypress-to-pixel; we adopt it.
- Dan Luu's terminal measurements: medians 5–45 ms across terminals, 99.9th
  percentile up to 110 ms. Tail latency is what users feel.
- Budget for backback (shell side, excluding Roslyn): keypress → frame **≤ 8 ms
  p50, ≤ 16 ms p99** (one frame at 60 Hz); cold start to editable **≤ 300 ms**;
  idle RSS **≤ 150 MB** without Roslyn. Roslyn adds 0.5–2 GB on large solutions
  whatever the shell; we report it separately and never hide it.

## 4. Text storage

- `crop` is faster than `ropey` in its own benchmarks (byte-indexed, like
  `String`); `ropey` is char-indexed, mature, used by Helix, and has UTF-16
  conversions (`char_to_utf16_cu`) that LSP needs. A third-party test found crop
  ~2× faster for multi-cursor edits but with higher memory after edits.
- Decision: **ropey** first (correctness of LSP positions matters more than the
  last 2×); swap behind `bb-text` if profiling says so.

## 5. Text rendering (for M1)
- `cosmic-text` (137 reverse deps; used by iced, GPUI, glyphon, Floem) vs
  `parley` (65; Bevy, Slint 1.14, Floem since 2026-03 moved to it). Both shape
  with HarfRust now. `glyphon` = cosmic-text + wgpu atlas; `wgpu_glyph` is dead.
- Whatever framework we pick, the text stack is one of these two. Not a risk.

## 6. The .NET project system
- Roslyn LSP needs `--stdio --logLevel <l> --extensionLogDirectory <d>`, then a
  `solution/open` notification; it signals readiness with
  `workspace/projectInitializationComplete` (nvim-lspconfig `roslyn_ls.lua`).
- Tests: two platforms. VSTest (legacy) prints a text list; MTP is opt-in via
  `global.json` `"runner": "Microsoft.Testing.Platform"` on .NET 10, uses
  JSON-RPC to the IDE, and `--list-tests json` is landing in the SDK (PR 54490).
  Mixed solutions are an error under MTP. We model the runner as an enum.

## 7. Lessons we take
1. Thin shell, fat servers. Never reimplement Roslyn, MSBuild or the debugger.
2. Layer the editor like Zed: text model → display map → render.
3. Keep the frontend behind a trait like Helix meant to (and GPUI pre-1.0 forces).
4. Synchronous core, async only at process edges (xi's lesson).
5. Measure from day 0 with Typometer-style keypress-to-pixel and RSS; publish
   numbers next to the claims (Zed's vendor page vs blogs shows why).
6. Licences decide the design: MIT everywhere, `vsdbg` excluded, `cargo deny` on.
