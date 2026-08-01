---
name: reference_description_facilities
description: The description language's provided names live in one table, and a new capability is a facility plus a bind pass in the app
type: reference
---

`apps/games/dsl/src/data/facility.rs` holds every name the description language
provides. One `Facility` carries its columns, its spawn bindings, its resource
blocks, its asset kinds, its asset sources, and its functions. `FACILITIES` is
the list, and the checker, the compiler, the interpreter, and the studio all
read it through the helpers at the foot of that file. Nothing else keeps a list
of provided names.

A function's opcode is its position in `facility::functions()`, and
`commands::builtin` dispatches on that number. Add a function at the end of the
`MATH` facility, never in the middle, or every compiled description shifts.

Adding a capability is four steps:

1. Add the names to a facility, or add a facility to `FACILITIES`.
2. Add a slot for each one to `Known` in `src/data/plan.rs` and fill it in
   `Maker::settle`. `Known` is how the runtime finds a column or a field
   without looking up a string each frame.
3. Write the bind in `src/systems/run.rs`, or a system of its own when it needs
   `&mut Storage` rather than `Commands`. `glow` binds lights, `dress` binds the
   parent and the visibility, `label` gathers the screen signs, and `show`
   writes the transforms.
4. Say what a rewind does to it. `Tape` holds 900 frames of `store.capture()`,
   and only what lives in the store comes back. A row that carries `Lifetime`
   counts down in the store, so the tape rewinds it. A prefab spawned for a
   `Model` row does not, so `respawn` queues it again.

Two shapes need a system of their own rather than a bind inside `tick`:

- A model. `spawn_prefab` wants `PrefabSpawn { storage, resources }`, which a
  parameter system holding `Commands` cannot give. `build` pushes the row and
  the path onto `store.pending`, and `systems::model::raise` spawns the prefab
  and parents it to the row's holder entity.
- The look. `systems::look::settle` holds `RenderSettings` and the sun's
  `Light`, which would take `tick` past the twelve extract parameters the ECS
  allows beside its queries.

The description names a file and never holds one. An `assets` block compiles
each row to the joined path as text, and the app reads it:
`assets Art from folder "assets/gltf"` with a row `helmet model
"DamagedHelmet.glb"` makes `Art.helmet` the text `assets/gltf/DamagedHelmet.glb`.
`systems::model::fetch` queues that path through `nightshade-load`, and `tick`
does not run until `Vault::ready` says every wanted model arrived or was lost.

Screen text is the retained UI, not the world text pass. A row that carries
`Label` spawns no mesh: `label` gathers it into `store.signs`, and
`systems::screen::paint` rebuilds one anchored UI tree whenever the signs
differ from the ones shown. `Size` is in pixels. World text through
`nightshade_text::Text` draws nothing in this tree, which
`apps/features/text_style` shows as well.

## The facilities and what each one costs

`place`, `paint`, `shape`, `sheet`, `mark`, `many`, `reach`, `walk`, `field`,
`dig`, `noise`, `spark`, `solid`, `look`, `screen`, `sense` and `math`. Each
names its columns with the shape each one holds, its spawn
bindings, its resource blocks, its asset kinds, its sources and its functions,
and the checker, the compiler, the interpreter and the studio all read that one
table.

Four of them reach a capability crate, and each cost one pass beside the table.

- `noise` names a sound off the disk. An asset row of kind `sound` becomes a
  path, `systems::sound::fetch` hands it to the audio loader under that path as
  its name, and a spawn carrying `Sound` gets an `AudioSource` and a lifetime.
- `spark` names a burst by preset. A spawn carrying `Burst` gets a
  `ParticleEmitter` and a lifetime. Slot zero of the particle textures has to
  carry a sprite or nothing draws, so `systems::spark::dress` writes a soft dot
  into it at startup.
- `solid` names what is solid. A spawn carrying `Body` gets a rigid body and a
  collider cut from its scale, a ball when the mesh is a sphere and a box
  otherwise. The engine then owns where that thing is, so `show` stops writing
  its transform and `settle` reads the transform back into the column before the
  systems run.
- Keeping and recalling a block writes every field of it to one file through
  `nightshade-save`, columnar rather than as records, merging with what is
  already there so two blocks do not overwrite each other.

Eight more reach a capability crate the same way.

- `paint` also names a picture. An asset row of kind `texture` becomes a name,
  a spawn carrying `Texture` puts that name on the material of the thing, and
  `Repeat` says how many times the picture crosses a face. The pictures are
  made at startup by `systems::skin::dress` rather than read off the disk, so a
  description carries no image file. A texture draws on the plain mesh path, so
  the instanced path that
  `reference_textured_material_needs_instanced_mesh.md` records is not needed
  here.
- `sheet` cuts pictures out of one atlas. A spawn carrying `Sprite` is a
  picture rather than a mesh and takes no material, and `Tile` picks one cell
  out of the named picture. `systems::sheet::dress` packs the atlas at startup.
- `mark` throws a picture onto what is already there. A spawn carrying `Decal`
  gets a `Decal` whose size is one unit, so the scale of the thing is the size
  of the mark and its depth.
- `many` draws one thing many times. A spawn carrying `Copies` gets an
  `InstancedMesh` with that many instances, each moved along by `Spread` from
  the one before, so a row of things is one entity and one draw.
- `reach` lets a person point at a thing. A thing that carries `Pick` can be
  picked, and `systems::point::reach` casts a ray from the pointer and writes
  `Picked` on the row it meets. `Picked` is a pulse, so it is gone by the next
  turn.
- `walk` lets a thing find its own way. `systems::path::bake` builds a navmesh
  out of every thing that carries `Ground` and cuts out every thing that is
  solid, whenever the shape of the store changes. A thing that carries `Path`
  gets an agent making for that point at its `Pace`, the engine owns where it
  is, and `settle` reads the transform back into the column the way it does for
  a body.
- `field` melts shapes together. A spawn carrying `Field` is not drawn as a
  shape of its own: it joins one signed distance field with its `Blend`, and
  `Carve` takes its shape back out of that field.
- `dig` gives a land of blocks. A `Land` block says the `seed` it grows from,
  how high its `hills` stand, the level its `shore` sits at and how far out to
  `reach`, and `systems::land` carries the `Dig` that fills one column from
  layered noise, the block table, the atlas it writes at startup and the viewer
  it takes off the camera. A thing that carries `Block` puts that kind of block
  where it stands and a thing that carries `Dig` takes one away, so a thing
  that moves writes a ribbon or cuts a trench. The viewer asks for every slab
  rather than the ones a walker could see, because the camera stands outside
  the land rather than in it, and `reach` has to cover the distance from the
  camera to the land or the near edge of the dug ground shows in the view.
  Switching descriptions resets the world, because `clear` puts `store.dug`
  back to false and the next turn hands `reset_the_world` to the queue.

One capability is not here. A portal pairs two things and shows what stands in
front of the other one, and the pairing, the plane and the viewer are
straightforward, but the scene draws nothing in the main view while the portal
eye cameras exist in this app, and the cause is in the backend rather than in
the facility.
