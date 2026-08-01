---
name: reference_template_escape_exit_trap
description: A game copied from the template does not quit on escape unless it registers escape_key_exit_system itself
type: reference
---

`template/src/lib.rs` does not register
`nightshade_platform::systems::escape_key_exit_system`, and a game copied from
the template therefore does not quit on escape.

A game that wants escape to close the window registers that system itself, one
line on `Stage::FrameUpdate`. A game with a pause screen registers nothing and
reads escape in its own system.

The system reads `is_key_pressed` rather than `just_pressed`, so the window
closes the moment the key goes down, and it closes cleanly, with code 0 and no
panic. To a player, and in a log, that reads as a crash.
