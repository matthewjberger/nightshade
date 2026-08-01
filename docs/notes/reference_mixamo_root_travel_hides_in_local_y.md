---
name: reference_mixamo_root_travel_hides_in_local_y
description: A Mixamo clip's forward travel sits in the hips bone's local Y after a Blender import, so a clip that looks in place still moves the character
type: reference
---

A Mixamo clip that travels carries its movement on the hips bone's translation
channel. After an import that orients bones automatically, the hips bone's local
axes are not the world axes, and the forward travel lands in local **Y**. Local
X and Z hold the sway and the small fore and aft shift.

This is a trap when a controller already moves the character. The clip's
translation adds to the controller's, and a travelling clip does not return to
where it started, so every loop snaps the character back. That reads as
rubberbanding.

Measure before believing a clip is in place, and measure every key against the
first, not the first against the last. A walk cycle can return to its start in
one axis while travelling in another:

```
walking   hips local Y, first key to last   165 units = 1.65 m over 1.03 s = 1.6 m/s
running   hips local Y, first key to last   294 units = 2.94 m over 0.70 s = 4.2 m/s
left_strafe   hips local X                  289 units = 2.89 m over 0.67 s = 4.3 m/s
```

Those numbers are walking speed, running speed and strafing speed, which is how
you recognize travel rather than a bob.

A pack is often mixed. In `assets/gltf/y_bot_locomotion.glb` the strafes and the
jump travel while the turns barely do, so testing one clip proves nothing about
the rest.

Two clips that each hold still can still move the character together. A blend
reads the weighted average of the hips position, so clips whose hips sit at
different places shift the body as the weights change. Hold every clip's hips at
one shared position, not at each clip's own first key.

When the controller owns movement, hold the hips translation constant on all
three axes in every clip. The vertical movement of a walk comes from the leg and
spine rotations, so the walk still reads correctly with the hips pinned.
`apps/features/locomotion` does this at load in `commands/blend_space.rs`, and
the asset is baked that way as well.

The other half of the rule is the opposite case: to let the clip drive the
movement instead, the controller must not translate the character at all, and
the engine's `root_motion` adds the hips delta to the entity without taking it
out of the pose, so the two would count it twice.
