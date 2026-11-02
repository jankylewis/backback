# BB-BUG-009 readback: the app with launchd's environment

Date 2026-11-03, by backback_mm. Launch as the Dock does: `env -i HOME USER
SHELL=/bin/zsh PATH=/usr/bin:/bin:/usr/sbin:/sbin open -n <app>` on a
scratch copy of the Debuggee solution (BB-TG-012).

| Build | C# server | Cmd+Shift+B / F5 |
|---|---|---|
| installed `55fac1d` | "Ready — 1 project failed to load: Debuggee.csproj" | F5: "Debug failed: debug I/O: No such file or directory" |
| bundle `bb-bug-009-dock-path` | "Ready" | Cmd+Shift+B: "Build succeeded: 0 warnings" |

F5 from the Dock also needs BB-BUG-008 (netcoredbg in backback's install
folder); with both merged it is read back on the installed app.
