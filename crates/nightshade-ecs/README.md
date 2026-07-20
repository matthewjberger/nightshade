# nightshade-ecs

An archetype-based Entity Component System with runtime component registration.
Entities that share a component set live together in contiguous, struct-of-arrays
storage, so iteration stays cache-coherent and parallelizes across cores on native
targets. Component types register at runtime rather than through generic
parameters, so a program composes its schema at startup instead of baking it into
types.

This is the ECS behind the [nightshade](https://crates.io/crates/nightshade)
engine, usable on its own.

```toml
[dependencies]
nightshade-ecs = "0.56.0"
```

## The dynamic world

`DynEcs` is a group of member worlds over one shared entity allocator. Component
types register into a world with the `dynamic_schema!` macro, which assigns each a
bitflag and emits the mask constants. Splitting components across member worlds
lets a group hold more than the 64 types one archetype table can index, and lets
independent subsystems share entity identity without sharing a schema.

The typed API routes by component type: `get`, `get_mut`, `set`, and `has` resolve
the member world that registered `T`; `query` and `query_ref` iterate typed tuples
with `.with` / `.without` / `.changed` filters; and `spawn_with((a, b, c))` derives
the archetype from the bundle. Resources are singletons in a per-group map, read
with `res` / `res_mut`. Tags are generation-checked markers stored in sparse sets,
so adding or removing one never migrates an archetype.

```rust
use nightshade_ecs::dynamic::DynEcs;

#[derive(Default, Clone)]
struct Position { x: f32, y: f32 }
#[derive(Default, Clone)]
struct Velocity { x: f32, y: f32 }

nightshade_ecs::dynamic_schema! {
    pub fn register {
        position: Position => POSITION,
        velocity: Velocity => VELOCITY,
    }
}
nightshade_ecs::impl_component!(Position, Velocity);

let mut ecs = DynEcs::default();
ecs.add_world_at(0, register());
ecs.spawn_with((Position::default(), Velocity { x: 1.0, y: 0.0 }));

ecs.query::<(&mut Position, &Velocity)>().for_each(|(position, velocity)| {
    position.x += velocity.x;
    position.y += velocity.y;
});
```

## What it provides

- Runtime component registration, with the archetype derived from the bundle at
  the spawn site
- Parallel iteration over Rayon on native targets
- Sparse-set tags that mark entities without fragmenting archetypes
- Change detection plus a cursor-based structural change log, so an incremental
  system like a render sync consumes only what moved this frame
- Sequence-numbered event channels with exactly-once cursor consumption
- A maintained hierarchy index over `ChildOf` links for descendant queries and
  recursive despawn
- Command buffers for structural changes queued during iteration
- A `Time` clock resource

## Features

`default = ["serde", "dynamic"]`

| Feature | What it adds |
|---------|--------------|
| `dynamic` | The dynamic world API: `DynEcs`, runtime registration, and the query and schema macros. Default |
| `serde` | Serde derives on the built-in types. Default |
| `snapshot` | Serialize a world and stream incremental deltas. Implies `dynamic` and `serde` |
| `state` | A state-machine layer over the scheduler |
| `raw_storage` | Direct table access for bulk visit-only passes |

## License

Dual-licensed under MIT or Apache-2.0, at your option.
