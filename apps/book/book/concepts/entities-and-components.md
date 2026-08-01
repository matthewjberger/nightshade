# Entities and components

There is one world, and it holds every row in the program: the renderer's, the
interface's, each composed capability's, and yours. Nothing declares a
component and nothing registers one.

## An entity is a number

An `Entity` is an identifier and nothing else. It carries no data, no methods,
and no behavior. It names a row, and the row is the set of components written
against that identifier.

```rust
let entity = world.spawn((
    Name("crate".to_string()),
    LocalTransform::default(),
    RenderMesh::new("cube"),
));
```

The archetype is derived from the component types you passed. There is no
registration step, no builder to finish, and no bundle type to declare.

## A component is a struct

Any `Send + Sync + Default + 'static` type is a component. The first write of
one appends it to the world's schema and takes the next of 256 mask bits, which
components and tags take from together.

```rust
#[derive(Clone, Copy, Default)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}
```

That is the whole declaration. No derive from the engine, no attribute, no
entry in a list somewhere else in the crate.

The rule that keeps a program readable is what goes where, and
[Systems and stages](systems-and-stages.md) covers the third line of it:

- Per-entity data goes in a component
- Data shared across entities goes in a resource
- Behavior goes in a system, never on the data

## Reading and writing

Typed access is by component type:

```rust
let health = world.get::<Health>(entity);
if let Some(health) = world.get_mut::<Health>(entity) {
    health.current -= 10.0;
}
world.set(entity, Health { current: 100.0, maximum: 100.0 });
let alive = world.has::<Health>(entity);
```

`get_mut` and `set` stamp change ticks. The renderer syncs by diffing those
ticks, so every render-facing component has to be written through a stamping
path or the change never reaches the screen.

## Tags cost a bit and nothing else

A marker with no data would still take a column and a migration as a component.
A tag takes a mask bit from the top of the same 256, and nothing else:

```rust
world.add_tag_type::<Selected>(entity);
world.remove_tag_type::<Selected>(entity);
let picked = world.has_tag_type::<Selected>(entity);
```

Tags are dropped when the entity is despawned, and a typed query can filter on
a tag set without naming the type as a component.

## Resources

A singleton lives in the resource map rather than on an entity:

```rust
fn count_frames(mut score: ResMut<Score>, time: Res<Time>) {
    score.elapsed += time.delta_time;
}
```

Both accessors panic when the resource is absent, so a capability whose plugin
was never composed fails loudly rather than quietly doing nothing.[^panic]

[^panic]: That is deliberate. A missing resource is a composition mistake, and
a composition mistake that reads as "the feature did nothing" costs far more
to find than one that stops on the first frame.

## A skin is components too

A skinned mesh is an entity with a pose on it, and the pose is a component the
animation writes and the renderer reads. Nothing about it is special.

![A fox walking on the spot, posed on the graphics card](images/posed-skins.jpg)

## What a parent carries

Attaching one entity to another is a component like any other. A child holds a
`ChildOf` naming its parent, and the pass that composes transforms walks the
tree and multiplies each child's own transform by its parent's.

Only the cube below is ever written to. The balls move because they are its
children, and the one you cut loose keeps the place it stood in, because
nothing is left to carry it.

```demo
id: hierarchy
title: A cube and what hangs off it
caption: Cutting a ball loose takes one component off it. Nothing else changes, and the ball stops where it was.
```

## Two of them at once

A page is not limited to one figure. Each one gets a tile of its own, its own
camera, and its own reading of the pointer, so dragging in one turns that
scene and leaves the others where they stand. The rows a demo spawns are
marked with a bit that only its own camera draws, which is how three scenes
stand in one world without seeing each other.

What they share is the sky, the post effects and the lights, because those are
one set for the whole frame rather than a thing an entity carries.

```demo
id: transform_parts
title: One cube, driven by hand
caption: Drag inside this figure and only this camera turns.
```

```demo
id: plugin_stack
title: Three behaviors over one ring
caption: The same page, a second scene. Its controls are its own and so is its camera.
```
