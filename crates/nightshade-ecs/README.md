# nightshade-ecs

An archetype-based Entity Component System in which a component is never
declared. Any `Send + Sync + Default + 'static` type is a component, and the
first write of one appends it to the world's schema and takes the next mask
slot, so a program never registers, marks, or lists its component types.
Entities that share a component set live together in contiguous,
struct-of-arrays storage, so iteration stays cache-coherent. Each table owns
one arena and each column is a typed span of it, so a row write is a store to
a known offset rather than a dispatch through a boxed vector.

This is the ECS behind the [nightshade](../nightshade)
engine, usable on its own.

```toml
[dependencies]
nightshade-ecs = "0.1.0"
```

## The world

`World` holds every component: contiguous `Vec<T>` columns per archetype, a
256-slot archetype mask, sparse-set tags, event channels, and
generational-handle liveness. A bundle is a flat set: a tuple of component
types, or a `bundle!` struct.

```rust
use nightshade_ecs::World;

#[derive(Default, Clone)]
struct Position { x: f32, y: f32 }
#[derive(Default, Clone)]
struct Velocity { x: f32, y: f32 }

let mut world = World::new();
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

## What it provides

- No declaration of any kind: the archetype comes from the bundle at the spawn
  site and a new type takes the next of 256 mask slots on its first write
- Sparse-set tags that mark entities without fragmenting archetypes, checked
  against a stale handle by generation
- Change detection, opt-in per world so a world that never diffs by tick pays
  nothing, exposed as `changed` / `added` query filters and a per-column
  high-water tick that skips untouched tables
- A cursor-based structural change log covering spawns, despawns, component
  moves, and tag flips, so an incremental system like a render sync consumes
  only what moved this frame
- Sequence-numbered event channels with two-frame lifetime and exactly-once
  cursor consumption
- A maintained hierarchy index over `ChildOf` links for descendant queries and
  cycle-tolerant recursive despawn
- Command buffers for structural changes queued during iteration
- `Res` / `ResMut` / `Query` / `Commands` / `EventReader` / `EventWriter`
  system parameters and a `Schedule` / `Stages` runner that reads the access a
  signature gives and batches the systems that cannot observe each other

## Features

`default = ["serde", "parallel"]`

| Feature | What it adds |
|---------|--------------|
| `serde` | Serde derives on the built-in types. Default |
| `parallel` | Runs the systems of a schedule that cannot observe each other on a rayon scope, one batch at a time. A system that reaches the whole world reports exclusive access and runs alone. Native only. Default |
| `profiling` | A tracing span around each system a schedule runs |

## Examples

```sh
cargo run --example basic     # queries, mutation, and marker tags
cargo run --example systems   # Res / ResMut / Query systems on a Schedule
cargo run --release --example parallel   # what a schedule groups into a batch
```

## License

Dual-licensed under MIT or Apache-2.0, at your option.
