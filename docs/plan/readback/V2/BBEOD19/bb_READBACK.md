# BBEOD19 readback: exception breakpoints

Date 2026-11-03, by backback_mm. Branch `bb-dap-exception-breaks`.

## What

Two ticks, as VS Code's Breakpoints view: **All Exceptions** (every throw,
caught or not) and **User-Unhandled Exceptions**, the filters the stock
netcoredbg advertises. They sit in the Debug menu (new; also Start,
Stop, the steps and Toggle Breakpoint) and on the Debug panel's title row
while a session lives; a click flips one, the status bar says
`Break on all exceptions: on`. The choice is saved per workspace
(`<state dir>/exceptions/<hash of the root>`, a versioned text file) and
sent at launch and on every change. A stop on an exception names it above
the call stack, in the error colour, and in the status bar:
`Paused on System.InvalidOperationException: total was 14 in MathOps.Fail()`.

## The adapter

Stock netcoredbg `initialize`: `exceptionBreakpointFilters` =
`user-unhandled`, `all`; `supportsExceptionInfoRequest` = true. Its
`exceptionInfo` reply for the Debuggee's caught throw (probe, 2026-11-03):

```
exceptionId  CLR/System.InvalidOperationException
description  Exception thrown: 'System.InvalidOperationException' in Debuggee.dll: 'total was 14'
details      fullTypeName System.InvalidOperationException, message total was 14,
             typeName InvalidOperationException, source Debuggee, stackTrace ...
```

The headline is built from `details` (BB-DAP-001): the live test went red
on the raw `CLR/...: Exception thrown: ...` text first.

## Result

| Check | Evidence |
|---|---|
| Capabilities decode the filters and the `exceptionInfo` flag | `capabilities_list_the_adapter_exception_filters` |
| Headline: full type and message from `details`; id without `CLR/` plus description without them | `the_headline_is_the_full_type_then_the_message_as_vs_code_shows_it` |
| Ticks toggle, list filters in adapter order, save and load per workspace, refuse an unknown file | `bb_exceptions::tests` (2) |
| Debug menu actions flip the ticks and the status bar says so | `the_debug_menu_actions_flip_the_ticks_and_say_so` (window) |
| A paused panel draws the exception row above the stack; a click on the All tick turns it on | `a_paused_panel_names_the_exception_and_its_ticks_take_a_click` (window, real click) |
| Real netcoredbg: `all` pauses on the Debuggee's caught throw in `Fail` at the throw line, named `System.InvalidOperationException: total was 14`; filters cleared, Continue runs to the end | `the_all_filter_pauses_on_a_caught_throw_and_names_the_exception` |
| Mutation: with `setExceptionBreakpoints` never sent, that live test fails (the program ends with no stop) | run on the branch before the gate |
| Real app (branch build, `env -i` with launchd's PATH, `BACKBACK_STATE_DIR` holding a saved All tick for the scratch solution): F5 builds, launches and pauses on the throw | `state`: `session=paused exception=Some("System.InvalidOperationException: total was 25") frames=["Calc.MathOps.Fail():39", "Calc.MathOps.Report():28", "Calc.Program.Main():10"] vars=["$exception={System.InvalidOperationException}", "total=25"]`; status `Paused on System.InvalidOperationException: total was 25 in Calc.MathOps.Fail()`; the editor on line 39 |
| Same run, F5 again: the caught exception lets the program finish | `Debugging ended: the program ended`, `session=none`, no netcoredbg process left |
| Installed app at `0b6406d` (`install.sh`, launchd's PATH), the app's own state folder holding the All tick for the scratch workspace: F5, then F5 | `Paused on System.InvalidOperationException: total was 25 in Calc.MathOps.Fail()`, frames `Fail():39`, `Report():28`, `Main():10`, `total=25`; then `Debugging ended: the program ended`, no netcoredbg left; 45 s launch to quit |

## Not covered

A click on the panel tick in the running app (the screen is locked, BB-TG-010);
the window test clicks the drawn tick through the same handler. The Debug
menu shows no check mark: gpui's menu is set once at launch, and the ticks
are per workspace; the status bar and the panel show the state.
