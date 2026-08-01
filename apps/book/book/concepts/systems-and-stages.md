# Systems and stages

A system is a function. The scheduler reads its signature to learn what it
touches, and runs it in the stage you registered it into.

## Naming what you touch

`add_system` takes the stage and the function. The parameters say the rest:

```rust
fn drain_health(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&mut Health, &Poisoned)>,
) {
    let delta = time.delta_time;
    query.for_each(|entity, (health, poison)| {
        health.current -= poison.rate * delta;
        if health.current <= 0.0 {
            commands.despawn(entity);
        }
    });
}
```

The available parameters are `Res<T>`, `ResMut<T>`, `Commands`, `Resources`,
`&mut Storage`, and up to six queries, or one query bundle. A system that
outgrows the slots declares a bundle with `query_bundle!`, which costs one
slot however many queries it holds.

A plain `fn(&mut World)` reaches everything and runs alone. Almost nothing
takes the world any more.

## The stages

Eight stages run in a fixed order, every frame:

| Stage | Runs | For |
| --- | --- | --- |
| `Startup` | Once | Spawning the world |
| `First` | Each frame | Work before the game |
| `Update` | Each frame | Game logic |
| `LateUpdate` | Each frame | Work after the game |
| `FrameStart` | Each frame | Engine, input and clocks |
| `FrameUpdate` | Each frame | Engine, animation and transforms |
| `FramePostUpdate` | Each frame | Engine, sync and culling |
| `Render` | Each frame | Publishing views |
| `Last` | Each frame | Reclaiming what nothing names |

Your work goes in `Update`. The engine stages belong to the plugins that
registered into them, and running one of those by hand is a mistake.

## Ordering

Within a stage, systems run in registration order unless something states
otherwise. To state an order, name both:

```rust
app.stages.stage_mut(Stage::Update.name()).order(
    std::any::type_name_of_val(&read_input),
    std::any::type_name_of_val(&move_player),
);
```

A system is named by its type name, so the constraint survives a rename of the
local binding and fails to compile when the function is gone.

The constraint is inert when the schedule does not hold both, and it is
applied by a stable topological sort on the first run. State an order rather
than splicing a system in by name.

## Mutating while you iterate

Adding or removing a component migrates archetypes, which conflicts with the
iteration in progress. The pattern is two passes:

```rust
let dead: Vec<Entity> = world
    .query_ref::<(&Health,)>()
    .iter()
    .filter(|(_, (health,))| health.current <= 0.0)
    .map(|(entity, _)| entity)
    .collect();

for entity in dead {
    world.despawn_recursive(entity);
}
```

Gather into a small `Vec`, then apply. `Commands` does the same thing for you
where a queue is a better fit.

## Screens

App-level states drive screen flow:

```rust
app.insert_state(Screen::Title)
    .on_enter(Screen::Playing, spawn_level)
    .on_exit(Screen::Playing, tear_down_level)
    .add_system(Stage::Update, while_in(Screen::Playing, drain_health));
```

A transition requested with `next_state` applies at the start of the next
frame, so a system never sees the world change under it.

## One pass, however many rows

A system does not know how many rows it will visit. The count below spawns and
despawns entities while the same pass runs over whatever is there, which is
what makes the number on the slider uninteresting to the code.

```demo
id: spawn_flock
title: A shell of cubes
caption: Moving the count spawns rows or despawns them. The pass that moves them is the same at one row and at a hundred.
```
