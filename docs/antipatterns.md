# Antipatterns

Patterns this repo does not want, and what to write instead. One entry for
each. Add to it when a pattern turns out to be worth naming.

## A component that is secretly an object

A component holds one value. When a struct gathers several fields under the
name of a thing, that struct is an object wearing a component's clothes, and
the ECS stops paying for itself.

```rust
pub struct Enemy {
    pub angle: f32,
    pub health: i32,
}
```

That is an object. The fields travel together whether or not a system reads
them, the name decides what the thing is, and nothing else in the world can
share `angle` or `health` without taking `Enemy` whole.

Write one component for each value:

```rust
pub struct Angle(pub f32);
pub struct Health(pub i32);
```

An enemy is then an entity that carries an `Angle` and a `Health`. The kind is
the composition, and a system asks for exactly the values it touches:

```rust
pub fn bleed(healths: Query<(&mut Health,)>) { }
pub fn turn(turners: Query<(&mut Angle, &mut LocalTransform)>) { }
```

### The name is not a component either

Replacing the struct with an empty marker moves the object rather than removing
it:

```rust
pub struct Enemy;
```

The entity is still named, and a query still asks "is this an enemy" instead of
"does this carry what I need". Select on the components an entity holds, and
use `Without<T>` where two kinds share a component:

```rust
pub fn drive(players: Query<(&Speed, &mut Angle, &mut Cooldown)>) { }
pub fn hunt(enemies: Query<(&Speed, &mut Angle), Without<Cooldown>>) { }
```

The player is the entity that carries a `Cooldown`, because the player is the
one that fires. Nothing declares it and nothing names it.

`apps/games/neon_static` is the live example.

### When a struct with several fields is still right

A resource is not a component. Data shared across entities lives in a resource
and keeps whatever shape it needs, so `Floor` holding a grid, a phase, and a
timer is correct. The same is true of the engine's own component types, which
carry a matrix or a texture reference as one indivisible value. The rule is
about per-entity data that a system would rather read one field of.

## A system whose signature names nothing

A system names what it touches. The schedule reads the access off the
signature, batches systems that do not conflict, and a reader learns what a
system reads without opening it. Two shapes say nothing.

### The world

```rust
pub fn drive(world: &mut World) {
    let save = world.res_mut::<Save>();
    world.query::<(&mut Angle,)>().for_each(|_entity, (angle,)| { });
}
```

A plain `fn(&mut World)` reaches every row and every resource, so it conflicts
with everything and runs alone. The nineteen that keep it run a schedule, which
`Schedule<H>::run(&mut H)` requires, and `CLAUDE.md` lists them.

### The resource map

```rust
pub fn shop_buttons(mut resources: Resources, mut buttons: Query<(&UiButtonData,)>) {
    let save = resources.res_mut::<Save>();
}
```

There is no whole-map parameter any more. The ECS carries `Res<T>`, `ResMut<T>`,
`Commands`, `&mut Storage` and `Query<...>`, and a signature that wants the map
does not compile.

### What to write

Name each resource as a parameter and the rows as a query:

```rust
pub fn shop_buttons(
    mut save: ResMut<Save>,
    mut interface: ResMut<Interface>,
    mut buttons: Query<(&UiButtonData,)>,
) { }
```

A system that wants more queries than the slots hold is two systems, or one
whose queries sit in a bundle declared with `query_bundle!`. A helper below a
system that the schedule never sees takes the `&mut Storage` and
`&mut ResourceMap` pair rather than a world, and a system that adds or removes
a component migrates archetypes, which is the one reason to stay plain.

### The table that forces the map

```rust
pub type Urge = fn(&ResourceMap, &Storage, Entity) -> f32;
```

A crate that takes answers from the game as a table of function pointers hands
each answer what it may read. An answer typed on `&ResourceMap` can read every
resource, so the system that calls it takes the whole world, and one type alias
puts the map back into a signature that named `Res<Time>` and nothing else. An
answer takes the rows and the entity:

```rust
pub type Urge = fn(&Storage, Entity) -> f32;
```

A value the answer needs that is not on a row is a component on an entity the
game keeps for it. `nightshade-tiles` keeps its `TileMap` and `OpenTile` that
way, so the `nightshade-cells` answers read the grid out of the rows.

### What still holds the map

Every engine, plugin and framework crate names what it reads, apart from
`run_camera_collision_sweep`, which hands the map to whatever capability
answers because that answer is `nightshade-physics` reading its own
`PhysicsWorld`, a resource with no entity to sit on.

Three parameters carry a group of resources so a caller names one thing rather
than seven: `PrefabParts` for what a prefab spawn writes, `EntityCaches` for
what an immediate despawn cleans up, and `TextureLoad` for what a texture upload
touches. All are system parameters, so a system that spawns, despawns or uploads
takes them and never the map.

What is left is the app that reaches the whole world inside one call: a table of
function pointers that builds a scene, a helper chain that spawns and then
mutates the rows it spawned, or a signature that would name more resources than
the parameter list can carry. Each of those takes `&mut World`, reports
exclusive access and runs alone, which is the shape for a system that reaches
everything. `act_on_the_prompt` in farm names seventeen resources, so it is two
systems before it is a conversion.

## A hand-rolled random generator

```rust
fn next_random(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*seed >> 16) as f32 / 65_535.0
}
```

Every app that wrote one wrote a different one, each with its own range, its
own bias, and a `u32` seed on a resource that every caller threads through.
`nightshade-random` is the framework crate every game draws from. Insert
`Random::seeded(seed)` as a resource and draw from it with the commands the
crate carries, `between`, `chance`, `index`, `pick` and `shuffle` among them.
A world seed that must differ on each run is the one roll that a seeded
generator cannot give, so that one roll comes from `fastrand::u32(..)` in one
function of the crate that owns the seed.

A hash that turns a coordinate into a stable value is not a generator and
stays, the way `hash_unit` in `apps/games/mashup` and `noise` in
`apps/games/crosstown` do.

## A wall of tuning constants

```rust
pub const ENGINE: f32 = 20.0;
pub const BRAKING: f32 = 28.0;
pub const MAXIMUM_SPEED: f32 = 32.0;
pub const GRIP: f32 = 7.0;
pub const DRIFT_GRIP: f32 = 1.3;
```

A constant is one number for every run of the program. Nothing can read the
set of them, nothing can show them on a panel, and nothing can keep a change
between runs, so the knobs a game is tuned by are hidden in a file nobody opens
at run time. A value a person is meant to change is declared once through
`nightshade-config`:

```rust
nightshade_config::config! {
    pub enum Settings {
        "Handling" {
            Engine: number(5.0, 40.0, 18.0),
            Grip: number(1.0, 15.0, 7.0),
        }
    }
}
```

The plugin inserts `Config<Settings>`, a system reads a value with
`config.number(Settings::Engine)`, `nightshade-ui-config` builds the panel from
the same declaration, and `persist` keeps the file. `apps/features/settings`
shows the whole of it, and `apps/games/gridlock` reads its handling, its
traffic and its night from one.

A constant is still right for what the program is made of: a table of model
names, a sprite index, an asset, and a structural limit the code cannot exceed,
a tile size, a buffer size, a maximum count. A framework crate carries none of
either kind, because it takes a settings resource the game inserts.

## A boolean flag standing in for a state

```rust
pub struct Player {
    pub flying: bool,
    pub watching: bool,
    pub gliding: bool,
}
```

The three flags name one thing, which is what the player is doing right now,
and a player is doing one of these at a time. As booleans they can all be true
together, so every reader guards against the pairs that must never happen and
every writer clears the other two by hand. The set of states and the moves
between them are the model, and here they are scattered through the reads and
writes that touch the flags.

A state is an enum, and the moves between its variants are a state machine. The
`stateless` crate writes both from one table:

```rust
stateless::statemachine! {
    transitions: {
        *Walk + Fly = Fly,
        Fly + Fly = Walk,
        Walk + Soar = Glide,
        Glide + Land = Walk,
    }
}
```

The macro gives a `State` enum and an `Event` enum, and `state.process_event(event)`
returns the next state or `None` when the move is not in the table. The player
holds one `State`, a reader asks `player.motion == Motion::Glide`, and a writer
says `player.motion.process_event(Event::Soar)` rather than setting a flag and
clearing its neighbors. A move that must not happen is absent from the table, so
the machine refuses it and no guard is needed.

`apps/games/blocker` is the live example, where the player's motion is a
`stateless` machine over walk, fly, watch and glide.
