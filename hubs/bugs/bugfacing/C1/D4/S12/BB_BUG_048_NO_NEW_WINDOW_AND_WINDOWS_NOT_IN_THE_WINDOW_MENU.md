# BB-BUG-048 — No New Window; open windows were not in the Window menu

id: BB-BUG-048
family: UIX
status: FIXED
date: 2026-11-05
signature: File menu without New Window; Window menu lists no workspace windows
source: headmate request 2026-11-05 (build 0501a5a: "can i open many backback in my machine? vs code allows this" -> both gaps: "both plz")
cause: not built: a new window came only from Open...; workspace windows had no title, and macOS lists only titled windows in the Window menu (gpui already sets it as NSApp's windowsMenu)
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open two folders with File > Open...; open the Window menu; look for File > New Window.

Expected: both windows listed under Window, by folder name; File > New Window (Cmd+Shift+N).
Actual: Window showed only Minimize and Zoom; no New Window.
