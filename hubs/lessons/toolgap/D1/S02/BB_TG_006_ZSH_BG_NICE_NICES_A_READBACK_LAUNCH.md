# BB-TG-006 — zsh `BG_NICE` nices an app a readback starts with `&`

id: BB-TG-006
for: BB-PRC-003
status: OBSERVED
date: 2026-11-02
source: `hubs/failures/failurereason/SYS/PRC/BB_PRC_003_TOOLS_SPAWNED_FROM_BACKGROUND_THREADS.md`

A readback that starts the app as `backback <path> &` in zsh gets a niced
app (`SN`), and every tool it spawns is niced too. Timings and throttling
read from such a run are not what a user sees.

Workaround: launch with `open -a <app> --args <path>` (env via `open --env`),
or `setopt NO_BG_NICE` in the launch shell; check `ps -o stat,nice` first.
