---
name: reference_pan_orbit_pitch_sign
description: A pan-orbit camera's pitch is positive above the focus, so a negative pitch puts the eye below the ground and the app opens looking up at the underside of the world
type: reference
---

`compute_pan_orbit_transform` in `nightshade-camera-controller` builds the eye
position as `focus + (yaw * Rx(-pitch)) * (0, 0, radius)`. A rotation of `-pitch`
about X sends `(0, 0, radius)` to a y of `radius * sin(pitch)`, so:

```
eye.y = focus.y + radius * sin(pitch)
```

**Positive pitch puts the camera above the focus. Negative pitch puts it
below.** The pitch limits are the full range, `-(PI/2 - 0.01)` thru
`PI/2 - 0.01`, so a negative value is accepted and nothing complains.

**How it fails when you get it wrong:** the app opens, the interface draws
correctly, and no gate fails. The picture is a wash of sky with a sliver of
ground at the top edge, because the eye is under the ground plane looking up.
A standing object reads as tilted or upside down, which sends you looking for
a transform bug that is not there. With `spawn_pan_orbit_camera_at(focus,
16.0, 0.6, -0.85, ..)` the eye sits 12 units below the floor.

**How to apply:** give an overhead camera a positive pitch. A three-quarter
view of a ground plane wants roughly `0.9` thru `1.1` radians, which is 50 thru
63 degrees above the horizontal. Read the number in an app you are copying
before you reuse it: a small negative pitch still puts the eye below, and a
scene whose subject floats above the ground can hide that.
