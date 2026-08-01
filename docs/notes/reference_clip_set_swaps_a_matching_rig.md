---
name: reference_clip_set_swaps_a_matching_rig
description: A ClipSet moves clips from one character to another whose rest pose already matches, and reports every joint it could not find
type: reference
---

A `ClipSet` in `nightshade-animation` holds clips that belong to a skeleton
rather than to the file they arrived in. It exists so a rigged character can be
swapped while the same clips keep playing.

```rust
let set = clip_set_from_player(player, "locomotion");
let bones = skeleton_bones(storage, hierarchy, root);
let report = clip_set_apply(target_player, &bones, &set);
```

`clip_set_apply` resolves every channel by `target_bone_name` against the new
skin's joints and ignores `target_node`. It matches on the part after the last
colon, so `mixamorig:LeftArm` and `LeftArm` reach the same joint. A channel with
no bone name, or one whose bone is absent from the new skeleton, is dropped and
counted.

The report is the point. `ClipSetReport` carries the number of clips, the
channels that resolved, the channels that were dropped, and the names of the
bones that were not found. `is_complete` is true only when nothing was dropped.
A swap onto a rig that does not match fails loudly rather than playing a T-pose
with no motion.

`clip_set_apply` also sets `resolve_by_bone_name` on the player. That switch
stops `resolve_target_entity` falling back to `node_index_to_entity`. Without
it the fallback still resolves, because the new model has nodes at those
indices, and every channel lands on whatever joint happens to sit there. The
fallback stays the default for a player whose clips came in with its own file.

**This is a swap between rigs that already match, not retargeting.** It moves
clips onto a second skeleton whose rest pose and inverse bind matrices agree
with the first. Two Mixamo characters agree. Two rigs from different sources do
not, and the same joint names over a different rest pose give a pose that is
wrong in a way the report cannot see. For a rig that does not match, the names
are only the first half of the problem.

`apps/viewer` is the live example. It holds the clips of the model it is about
to replace, and `Take clips` in the `+ Add` panel puts them on the skeleton that
is open now. The agent carries the same thing as `apply_clips`.
