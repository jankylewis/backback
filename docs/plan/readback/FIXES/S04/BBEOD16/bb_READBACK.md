# BBEOD16 readback: file type icons in the Explorer

Date 2026-11-03, by backback_mm. Branch `bbeod-16-icons`.

## What

13 new icons for files that are not C#, Markdown, a project or a solution:
JSON, XML (`.config`, `.resx`, `.xaml`, `.runsettings`), MSBuild (`.props`,
`.targets`, other project files), YAML, Razor, SQL, shell, JS/TS, HTML/CSS,
images, config (`.gitignore`, `.editorconfig`, `.env*`, `.toml`), Docker,
text. Mapping after VS Code's default icon theme; icons drawn for backback
(20 x 20 line art, `currentColor`, tinted from theme tokens).

## Result

`ICONS.png`: each icon rendered with its dark-theme tint on the editor
background (Quick Look renderer, strokes included), next to the existing
`.cs` and `.md` icons. Tests: 32 file names map to their type, unknown
names stay generic, and every type has its own valid SVG that takes the
theme tint.

## Not covered

An Explorer screenshot of the running app: the screen was locked
(BB-TG-010). The row view's icon call is unchanged; only the kind of a
non-C# file now picks its own icon and tint.
