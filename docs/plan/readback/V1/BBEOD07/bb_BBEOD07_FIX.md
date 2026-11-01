# BBEOD07-FIX readback, 2026-11-01

- Seat: `backback_2`; branch: `bbeod-07-release`.
- Family: the release wrapper can name and sign a valid binary that is not the GUI.
- Known instance: the first bundle used `bb` as `CFBundleExecutable`.
- Guard: bundle creation copies and compares `backback` before signing.
- The served check reads `CFBundleExecutable`, opens the `.app`, and matches
  the visible window to this bundle's process ID.
- The pre-fix check exited 1 at the executable assertion.
- The fixed check reported `bundle executable: backback GUI binary`,
  a visible window for this process, and `leftover processes: 0`.
- [WINDOW.png](WINDOW.png) is a capture of that process's window.
  The Explorer shows a scratch C# fixture. The app quit through
  `NSRunningApplication.terminate()` afterward.
- `codesign --verify --deep --strict` and `plutil -lint` passed in the bundle script.

## Boundary observed

- Launch Services opened the scratch folder and opened the app without an argument.
- Launch Services with the repository fixture under `Documents` left the
  process alive without a window. Direct execution with that fixture opened a
  window. This points to a macOS access-context difference; its cause is
  unverified. The served gate uses a scratch fixture for an honest window check.

## Icon

- The mascot had warm dark hair, but the fringe retained a pointed wedge.
- The strap moved left. The pack mark still showed two chevrons without
  the intended slash. BBEOD07-MERGE corrected the mark and fringe.
- The `.icns` archive contains 10 PNG sizes. BBEOD07-MERGE regenerated
  [the contact sheet](../../../../release/bb_assets/bb_mascot/previews/CONTACT_SHEET.png)
  to show all 10 on dark.
