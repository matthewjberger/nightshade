# nightshade-ecs

An archetype-based Entity Component System with runtime component registration.
Entities that share a component set live together in contiguous, struct-of-arrays
storage, so iteration stays cache-coherent and parallelizes across cores on native
targets. Component types register at runtime rather than through generic
parameters, so a program composes its schema at startup instead of baking it into
types, and zero `unsafe` unless the `raw_storage` backend is turned on.

This is the ECS behind the [nightshade](https://crates.io/crates/nightshade)
engine, usable on its own.

```toml
[dependencies]
nightshade-ecs = "0.56.0"
```

## The dynamic world

`DynWorld` registers component types at runtime and keeps contiguous `Vec<T>`
columns per archetype, `u64` component masks, sparse-set tags, event channels,
and generational-handle liveness. Types register lazily on first use, so most
programs never register anything by hand. A type marked once with
`impl_component!` is a bundle, so components, tuples, and `bundle!` structs nest
and flatten into a single spawn.

```rust
use nightshade_ecs::dynamic::DynWorld;

#[derive(Default, Clone)]
struct Position { x: f32, y: f32 }
#[derive(Default, Clone)]
struct Velocity { x: f32, y: f32 }

nightshade_ecs::impl_component!(Position, Velocity);

let mut world = DynWorld::new();
world.spawn((Position::default(), Velocity { x: 1.0, y: 0.0 }));

world.query::<(&mut Position, &Velocity)>().for_each(|_entity, (position, velocity)| {
    position.x += velocity.x;
    position.y += velocity.y;
});
```

Borrow mutability comes from the tuple, and mutable elements stamp change ticks
per visited entity. Queries take up to eight elements, `Option<&T>` for optional
presence, and `.with` / `.without` / `.changed` / `.added` filters. A
single-element query skips the tuple (`query::<&mut Position>()`). On a shared
borrow, `query_ref` runs read-only tuples as a real iterator whose items borrow
the world, so results collect and compose with adapters. Typed access
(`get`, `get_mut`, `set`, `has`) resolves the owning column by type. Resources
are singletons read with `res` / `res_mut`, and tags are generation-checked
markers stored in sparse sets, so adding or removing one never migrates an
archetype.

## Grouped worlds

`DynEcs` groups several `DynWorld` members over one shared entity allocator. It
is the way past the 64-component ceiling of a single mask: each member carries
its own registry and full mask space, one entity can hold rows in any
combination of members, and despawning retires it everywhere behind the same
generation broadcast, so a stale handle is refused in every member. Declare the
members once with `dynamic_worlds!`, which emits the index constants and the
build function, and extend a built group with `add_world_at`.

```rust
use nightshade_ecs::dynamic::DynEcs;

nightshade_ecs::dynamic_schema! {
    pub fn register_core {
        position: Position => POSITION,
        velocity: Velocity => VELOCITY,
    }
}

let mut ecs = DynEcs::new();
ecs.add_world_at(0, register_core());
ecs.spawn_with((Position::default(), Velocity { x: 1.0, y: 0.0 }));
```

A component type lives in exactly one member, so the group routes typed access
itself: `ecs.get::<T>()`, `set`, `query`, and `query_ref` resolve the owning
member per type, and `ecs.spawn_with(bundle)` routes each component of a bundle
to its member, so one bundle can span members. Tuples whose components live in
different members run through `ecs.query_join`: the member holding every mutable
element drives at slice speed, the others resolve per entity read-only. The
group carries its own resource map and event channels for state that crosses
member boundaries, marker tags that spend no member's mask bits
(`ecs.add_tag_type::<Selected>()`, `tag_set_type` to filter a member query), and
`ecs.step()` expires group events and steps every member in one call.

## What it provides

- Runtime component registration, with the archetype derived from the bundle at
  the spawn site
- Parallel iteration over Rayon on native targets (`par_for_each`)
- Sparse-set tags that mark entities without fragmenting archetypes, checked
  against a stale handle by generation
- Change detection, opt-in per world so a world that never diffs by tick pays
  nothing, exposed as `changed` / `added` query filters and a per-column
  high-water tick that skips untouched tables
- A cursor-based structural change log covering spawns, despawns, component
  moves, and tag flips, so an incremental system like a render sync consumes
  only what moved this frame
- Sequence-numbered event channels with two-frame lifetime and exactly-once
  cursor consumption, at both the world and group level
- A maintained hierarchy index over `ChildOf` links for descendant queries and
  cycle-tolerant recursive despawn
- Command buffers for structural changes queued during iteration
- Serializable snapshots and incremental deltas of a world or a whole group,
  behind the `snapshot` feature
- `Res` / `ResMut` / `Query` system parameters and a `Schedule` / `Stages`
  runner that composes systems from independent parts
- A `Time` clock resource

## Features

`default = ["serde", "dynamic"]`

| Feature | What it adds |
|---------|--------------|
| `dynamic` | The dynamic world API: `DynWorld`, `DynEcs`, runtime registration, and the query and schema macros. Default |
| `serde` | Serde derives on the built-in types. Default |
| `snapshot` | Serialize a world or group and stream incremental deltas. Implies `dynamic` and `serde` |
| `state` | A state-machine layer over the scheduler, gating systems by a current-and-next value |
| `raw_storage` | A contiguous-byte column backend behind an identical public API, for the tightest per-table loops |

## License

Dual-licensed under MIT or Apache-2.0, at your option.
