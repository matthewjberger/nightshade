# nightshade-near

An entity that carries `Bulk` joins a proximity grid, an entity that also carries `Crowded` is held apart from its neighbors, and a game asks what stands near a point under a tag it owns. The crate names no GPU API. It gathers the points and the asks, and the render backend builds the grid, answers the asks and works out the pushes in a compute pass, then copies the answers back without blocking. The answers a system reads are the answers to the asks of the frame before, the way a GPU pick comes back after the click.

## Composing it

```rust
app.add_plugin(nightshade_near::NearPlugin::default())
    .run()
```

`NearPlugin` inserts `NearSettings`, `NearFrame`, `NearAsks` and `NearAnswers`, and registers the gather on `Stage::Render`. The settings hold the cell size of the grid, the room the crowd keeps between bodies, and the capacity of one cell. A program that wants different numbers builds the plugin with its own settings.

## Using it

```rust
use nightshade_near::prelude::*;

pub fn spawn(mut commands: Commands) {
    commands.spawn((LocalTransform::default(), Bulk(0.5), Crowded(1.0)));
}

pub fn look_around(hero: Res<Hero>, mut asks: ResMut<NearAsks>) {
    ask_what_is_near(&mut asks, 1, hero.place, 6.0);
}

pub fn count_them(answers: Res<NearAnswers>, mut readout: ResMut<Readout>) {
    readout.near = counted_near(&answers, 1);
    if let Some((entity, distance)) = nearest_of(&answers, 1) {
        readout.nearest = Some(entity);
        readout.reach = distance;
    }
}

pub fn keep_apart(answers: Res<NearAnswers>, mut bodies: Query<(&mut LocalTransform,)>) {
    bodies.for_each(|entity, (transform,)| {
        transform.translation += push_of(&answers, entity);
    });
}
```

`Bulk` is the radius a body takes in the grid, and `Crowded` is the weight of the push the crowd gives it. `ask_what_is_near` puts one ask under a tag and replaces the ask that tag held before, and `stop_asking` takes it away. `answer_of` reads the whole answer for a tag, `counted_near` the number of bodies in the radius, `nearest_of` the nearest body and how far it stands, and `push_of` the push a crowded body gets this frame. `apps/features/near` is the live example.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-near = { path = "../nightshade-near", default-features = false, features = ["contract"] }
```

The contract carries `Bulk`, `Crowded`, `Ask`, `Answer`, `NearFrame`, `NearAsks`, `NearAnswers` and `NearSettings`, and it takes only the ECS for the entity type, which is how the render backend reads the frame and writes the answers without naming the plugin.
