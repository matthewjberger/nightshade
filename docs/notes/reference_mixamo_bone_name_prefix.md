---
name: reference_mixamo_bone_name_prefix
description: Mixamo glTF bones carry a mixamorig prefix and the engine stores the raw name, so resolve bones by suffix
type: reference
---

Mixamo glTF/GLB characters (e.g. `assets/gltf/x_bot.glb`) name every skeleton joint with a `mixamorig:` prefix: `mixamorig:Hips`, `mixamorig:LeftUpLeg`, `mixamorig:LeftArm`, `mixamorig:LeftForeArm`, `mixamorig:LeftHand`, etc. The colon is a literal character. x_bot has 65 joints + `Beta_Joints`/`Beta_Surface`/`Armature`, one skin, and its two bundled clips (`mixamo.com`, `mixamo.com.001`) are STATIC reference/T-poses (no motion) so `apps/games/sandbox` authors x_bot's motion procedurally in `systems/world/pose.rs` rather than playing the clips.

The engine stores the RAW glTF node name verbatim in both the `Name` component and `AnimationChannel.target_bone_name` (`commands/gltf/animation.rs` in `nightshade-prefab` reads `target_gltf_node.name()` and strips no prefix). So any lookup by the plain name (`"LeftArm"`, `"Hips"`) fails against `bone_name_to_entity`.

When that goes wrong an authored clip / IK chain using plain bone names finds nothing in `resolve_target_entity`, falls through to `target_node: 0`, and every channel lands on node 0 while IK chains resolve to nothing. Result is a clean T-pose with no animation AND no IK looks like a broken pose pipeline but is just unresolved names. Fix: resolve bones by SUFFIX (`name.ends_with("LeftArm")`) so it works prefixed or plain, or use the full `mixamorig:`-prefixed string.
