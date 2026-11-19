# BBEOD37 — binary size by crate, and first frame on the dna-ax tree, 2026-11-20

Apple M4 Pro (12 cores), macOS 27, rustc 1.99.1. Release profile as
shipped (`lto = "fat"`, `codegen-units = 1`, `strip = true`). Base `aa1d223`
(before the picker), master `243c495` (BBEOD36 merged). No code changed.

## 1. Binary size (BB-PRF-008)

Method: `cargo build --release -p backback -p bb` as `bb_bundle.sh` runs it,
then `cargo rustc --release -p backback --bin backback -- -C
link-arg=-Wl,-map,<file>`: the same binary (base 14,294,192 B both ways,
master 16,144,656 B both ways) plus the linker's map of every symbol and
the object it came from. Rust symbols (v0 mangling) count for the crate
that defines them; C objects for their archive. `cargo bloat` cannot see
symbols in a stripped binary, the map can.

| Binary | `aa1d223` | `243c495` | change |
|---|---:|---:|---:|
| As linked (linker ad-hoc signature, 4 KB pages) | 14,294,192 B | 16,144,656 | +12.9 % |
| After `codesign --force --sign -` (as in the bundle) | 14,211,216 | 16,050,928 | +12.9 % |
| BBEOD36 bundles (`aa1d223` / fix on `19dd6d3`) | 14,227,712 | 16,050,992 | +12.8 % |

The signed rebuild matches BBEOD36's bundles to one 16 KB page (base) and
64 B (master). By section (as linked): `__TEXT,__const` +1,532,496 B
(read-only tables), `__TEXT,__text` +268,672 (code), `__DATA_CONST,__const`
+10,456, `__TEXT,__cstring` +7,312.

Per crate (bytes of map symbols):

| Crate | `aa1d223` | `243c495` | change | share |
|---|---:|---:|---:|---:|
| tree-sitter-rust grammar (C `parser.c` + `scanner.c`) | 0 | 1,118,621 | +1,118,621 | 61.4 % |
| tree-sitter-python grammar (C) | 0 | 454,241 | +454,241 | 24.9 % |
| backback (desktop) | 644,780 | 724,512 | +79,732 | 4.4 % |
| gpui (generic code built for our types) | 870,500 | 904,612 | +34,112 | 1.9 % |
| core | 918,611 | 946,327 | +27,716 | 1.5 % |
| bb-intel | 48,444 | 74,560 | +26,116 | 1.4 % |
| serde_core | 224,732 | 244,244 | +19,512 | 1.1 % |
| bb-cargo (new) | 0 | 16,172 | +16,172 | 0.9 % |
| LTO locals (unnamed constants) | 1,131,891 | 1,145,092 | +13,201 | 0.7 % |
| bb-lang (new) | 0 | 12,284 | +12,284 | 0.7 % |
| tokio / bb-python (new) / serde_json / alloc | | | +9,676 / +9,384 / +8,396 / +7,840 | 1.9 % |
| tree-sitter runtime / C# grammar | 126,187 / 5,330,309 | 128,399 / 5,329,053 | +2,212 / -1,256 | 0.1 % |
| std / bb-lsp | 201,964 / 53,776 | 191,368 / 40,824 | -10,596 / -12,952 | -1.3 % |
| all other crates (net) | | | +6,381 | 0.4 % |
| total of map symbols | 14,086,493 | 15,907,285 | +1,820,792 | 100 % |

The grammar archives' own objects agree (`size -m` of the build's
`libtree-sitter-rust.a`: `__TEXT,__const` 1,088,794 B; Python 434,380 B).
backback's own crates together: +129,332 B (7.1 %).

Safe win: none found. `Cargo.lock` gained 5 packages (bb-lang, bb-cargo,
bb-python, tree-sitter-rust 0.24.2, tree-sitter-python 0.25.0); no version
moved and the duplicated names stay the same 49. The `fixtures` features
of bb-lang, bb-cargo and bb-python are dev-dependencies only (resolver 2),
so no test data reaches the release binary. 86 % of the growth is the two
grammars' parse tables, which Rust and Python colours need. Nothing changed.

## 2. First frame on the dna-ax tree (BB-PRF-009)

Bundles: BBEOD36's `aa1d223` and fix bundles (the fix's release code is
`243c495`'s). Dock-like direct launch, own empty state folder, `--no-write`,
1 warm-up + 5 samples, interleaved, median. Load 2.2-3.0 (other fleets'
idle `dotnet` processes; not idle). Own pid stopped by TERM; no app or
child left after any run (checked each run).

| Case | `aa1d223` | fix | change |
|---|---:|---:|---:|
| No saved choice (first open): exec to first frame | 184 ms | 350 | +90 % REGRESSION |
| Same, the app's own `first_frame_ms` | 56 ms | 227 | +171 ms |
| Saved choice C# + Python: exec to first frame | 184 ms | 184 | 0 % ok |
| Same, the app's own `first_frame_ms` | 56 ms | 56 | ok |

Samples, first open: base 184 181 187 190 180, fix 350 406 374 342 344.

Profile (release program on `bb_lang`, warm cache, median of 5-7):
`detect` 229 ms; the walk reads 13,178 entries in 4,873 folders, so it
costs ~47 µs a folder (open, read, ignore files), not per file (50,000
`.cs` in a few folders: 74 ms). Without git ignores it would read 115,091
entries (1,469 ms): the ignores are what keep it short. Same walk on 4
threads (`build_parallel`): 68 ms.

Not changed (ruling asked): the early walk (BB-PRF-007) hides only the
first ~56 ms; the window waits ~170 ms more on the first open of a tree
with many folders. A saved choice ends the walk early, so later launches
equal base.
