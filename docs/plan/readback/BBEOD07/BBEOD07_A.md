# BBEOD07-A readback, 2026-11-01

- Seat: `backback_2`; branch: `bbeod-07-release`.
- Built `backback.app` from this tree with `bb_bundle.sh`.
- `codesign --verify --deep --strict` passed. The signature is ad hoc.
- `Info.plist`: bundle id `io.github.jankylewis.backback`, version `0.0.1`, minimum macOS `13.0`.
- The bundled `bb doctor` exited 0: dark theme, .NET SDK 10.0.403, Roslyn found.
- `netcoredbg` was reported optional and absent.
- Bundled `bb scan` read the fixture solution: 4 projects, 2 test projects.
- Bundled `bb scan` read MbTx read-only: 17 projects, 11 test projects.
- The PNG previews have measured dimensions 16, 32, 128, and 1024 px.
- Polish check: the hair edge stays above the goggles, and the pack strap has a distinct dark border.
- At 16 px, the head, goggles, and pack remain visually separate.
- M0 has no GUI, so app screenshots from readback do not exist yet.

## Lesson

The bundle is a distribution wrapper for the current CLI. Validate its actual
executable and signature, and show the CLI limit in install instructions.
