---
name: reference_active_camera_and_screen_states
description: CameraControllerPlugin drives whichever camera is active, and a transition into the current state fires no on_enter
type: reference
---

Two traps in a game that drives its own camera and its own screens.

**CameraControllerPlugin moves the active camera.** Its frame systems
(`look_camera_system`, `fly_camera_system`, the orbit and third-person
ones) read `ActiveCamera` and write that camera's `LocalTransform`. There is no
marker component to opt out, and they work even with `Camera.smoothing` set to
`None`. A game that drives its own camera must leave the plugin uncomposed and
call `first_person_camera_look_system` from its own system. With the plugin
composed, WASD flies the map camera and a right drag turns it.

**A transition into the state that is already current fires no event**, so
`on_enter` never runs and an in-place rebuild of a screen must be driven by a
flag in the game's own resource, read by a system under `while_in`.

Related: [[project_boomstick_app]]
