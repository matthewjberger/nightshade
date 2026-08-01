---
name: project_blackmarrow_app
description: apps/games/blackmarrow is the Penumbra-style indoor horror app, where one hold-and-pull mechanic drives loose objects, hinged doors and sliding drawers through joints
type: project
---

`apps/games/blackmarrow` is a first-person survival horror game in a pumping
station under the ice, in the line of Penumbra: Overture and Amnesia: The Dark
Descent. It is the app to read for physical hand interaction and for an indoor
scene lit only by its own lamps.

The hand is one mechanic and not three. `systems/hand.rs` casts one ray from
the eye, takes the hit point into the body's local frame, and each frame pushes
the body toward `eye + forward * hold_distance` with
`physics_world_apply_impulse_at_point` at that grip. What the body can do with
that push is the joint's business: a door leaf is on a `RevoluteJoint` about Y
with symmetric limits, a drawer tray is on a `PrismaticJoint` along Z with a
travel limit, and a loose object has no joint and gets gravity added back into
the pull so it hovers. Adding a new fixture is a joint and a `Handle`
component, not a new system.

A locked door is a static leaf carrying `Locked(Some(kind))`. Turning the key
despawns it and calls `station::hang_leaf` for the same doorway, because rapier
joint limits cannot be changed after the joint is made and a swap needs no
handle.

The station is hand-placed, not a tile grid. `data/rooms.rs` carries six room
boxes, twenty-one wall runs and five doorways, and `station::raise_wall` splits
each run around the doorways that sit on it and puts a lintel over each
opening. `data/route.rs` carries the fifteen waypoints the wolf walks, and
`queries::next_hop` is a breadth-first search over them, so the wolf never
walks through a wall.

The wolf is a kinematic body moved with `physics_world_set_translation`, with
the skinned Quaternius model as a child whose rotation the game writes, because
the physics sync writes the body's transform and there is no call to set a
body's rotation.

It carries no fill light on purpose. Refer to
[the volumetric fog note](reference_volumetric_fog_lets_a_shadowless_sun_through.md).

Its assets are the Screaming Brain Studios horror textures, the Kenney
furniture kit and one Quaternius wolf, all CC0.

The entry hall carries the fixtures the rest of the station does not: a caged lamp on a spherical joint that swings and throws the shadows of its own bars, a tool rack of rigid bodies built from primitives over one collider each in `data/tools.rs`, an emissive glowstick that carries a point light so it lights what you hold it against, and a page on the desk that opens a `Reading` screen over a menu panel, with its text in `data/notes.rs`. The wolf's bite goes through `Config<Settings>` from `nightshade-config`, where `Invulnerable` is on by default and persists in `blackmarrow.toml`. The developer console from `nightshade-shell` carries `noclip`, which turns the character controller's gravity and engine input off and moves the body by hand, and every system that reads a key steps aside while the console is open.
