---
name: reference_heading_yaw_turns_left
description: In the nightshade world an increasing yaw turns a (sin, cos) heading to the LEFT, so steering must subtract
type: reference
---

The engine world is right-handed, Y up, and a camera looks down its local -Z
(`camera_look_rotation` builds yaw as `(-direction.x).atan2(-direction.z)`).

For a viewer looking along `forward` with +Y up, screen right is
`cross(forward, up)`, which is `(-forward.z, 0, forward.x)`.

A character that carries `heading = (yaw.sin, 0, yaw.cos)` and rotates with
`quat_angle_axis(yaw, Y)` has its model's local +Z on that heading. But the
derivative of that heading is `(yaw.cos, 0, -yaw.sin)`, which points to the
viewer's **left**. So an increasing yaw is a left turn.

Therefore a steering value where +1 means right must be **subtracted** from the
yaw. One sign at the yaw corrects the keyboard, the mouse and the gamepad at
once, so do not negate each input device. See [[project_hairpin_app]].
