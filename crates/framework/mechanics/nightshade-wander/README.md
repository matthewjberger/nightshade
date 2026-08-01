# nightshade-wander

The walk an idle creature takes around the place it calls home. Each wanderer picks a spot inside a disc around its home, walks across the play plane toward it, rests for a rolled while when it arrives, and then picks again. The whole loop runs off four components and one settings resource, so a village of creatures looks busy without any of them being told where to go.

## Composing it

```rust
use nightshade::app::App;
use nightshade_wander::{WanderPlugin, WanderSettings};

fn build(app: &mut App) {
    app.add_plugin(WanderPlugin {
        settings: WanderSettings {
            reach: 6.0,
            speed: 1.4,
            arrival: 0.2,
            rest_low: 0.5,
            rest_high: 3.0,
        },
    });
}
```

The plugin inserts `WanderSettings`, makes sure a `Roll` and a `PlayPlane` are there to be read, and registers `wander` on `Stage::Update`, which is the one writer of the wanderer's transform and its rest. Nothing is announced, because the state a game wants is already on the entity.

## Using it

```rust
use nightshade_wander::commands::{pick_a_spot, rest_a_while};
use nightshade_wander::queries::{arrived, resting};
use nightshade_wander::{Home, Resting, Roaming, Wandering};

storage.set(walker, Home(spot));
storage.set(walker, Wandering(spot));
storage.set(walker, Resting(0.0));
storage.set(walker, Roaming(1.0));

let waiting = resting(&rest);
let there = arrived(plane, position, &wandering, &settings);
```

All four components have to be present for an entity to wander. `Home` is the middle of the disc the walk stays inside, `Wandering` is the spot being walked to now, `Resting` counts down the pause and holds the walker still while it is above zero, and `Roaming` is the share of the settings speed this creature moves at. `pick_a_spot` rolls a new target on the disc of `reach` around the home, and `rest_a_while` rolls a pause between `rest_low` and `rest_high`, and both are called by the frame system as the loop comes round.

## Driving it

```
just example wander
```

Space cuts every walker's rest short and rolls it a fresh spot, and R sends the whole field to rest for a rolled while. Q and E lower and raise the share of the speed they roam at, so the field crawls or hurries, and the readout counts the walking against the resting and gives the share.
