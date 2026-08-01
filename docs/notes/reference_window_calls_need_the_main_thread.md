---
name: reference_window_calls_need_the_main_thread
description: A window call from a parameter system can deadlock the whole frame, because the platform routes it to the thread that owns the event loop while that thread waits inside the batch, so a system that calls the window names the MainThread parameter
type: reference
---

A windowing platform routes a call on `window.handle` to the thread that owns
the event loop and blocks until that thread pumps messages. The parallel
schedule spawns each system of a multi-entry batch into a `rayon::scope`, and
the thread that owns the event loop is the one waiting on that scope.

**How it fails when you get it wrong:** the call blocks on a thread that is
itself blocked, and nothing breaks the cycle. The window stops answering, the
title bar reads "Not Responding", and the last drawn frame stays on the screen
with a live frame counter frozen on it, so it reads as a render hang rather
than a scheduling one. It bites only when the call actually runs, so a cursor
grab that turns on when a player enters the world leaves the menus working and
freezes the game one frame later. Sampling the process settles it: a deadlock
burns no CPU time and holds no running thread, while a spin burns a core.

**How to apply:** a command that wants a window change records the intent on
the `Window` resource and does not call the platform. `nightshade-platform`
carries the requested and applied pairs (`cursor_locked` beside
`applied_cursor_locked`) and one system in `Stage::Last` applies the difference.
A system that must call the window itself names `MainThread` as its first
parameter, which reports main-thread access and gives that system a batch of
one. `just lint-dag` fails on a file that calls a method on `window.handle` and
is not recorded in `tools/dag-check/window-handle.txt`, so a new call site has
to be looked at rather than found by a player.
