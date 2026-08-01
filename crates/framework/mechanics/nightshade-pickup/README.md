# nightshade-pickup

`nightshade-pickup` is the thing lying on the ground and the character that reaches it. It measures the gap across the play plane between every collector and every pickup that is still there, marks the ones that were reached, and says so. It holds no idea of what a pickup is worth, so a coin, a health orb and a key are the same mechanic with different kinds.

## Composing it

```rust
app.insert_resource(nightshade_plane::PlayPlane(nightshade_plane::Plane::Ground));
app.add_plugin(nightshade_pickup::PickupPlugin);
```

The plugin inserts `PlayPlane` when the game has not already, and registers `gather_the_pickups` on `Stage::LateUpdate`, which is where the reach settles for the frame. It announces `PickedUp`, carrying the collector, the pickup, the kind and the count.

## Using it

```rust
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_pickup::{Collector, PickedUp, Pickup, PickupCount, Taken};

storage.set(coin, Pickup(COIN_KIND));
storage.set(coin, PickupCount(1));
storage.set(coin, Taken(false));
storage.set(walker, Collector(1.5));

pub fn take_the_coins(taken: EventReader<PickedUp>, storage: &mut Storage) {
    let gone: Vec<_> = taken.iter().map(|word| word.pickup).collect();
    storage.despawn_entities(&gone);
}
```

`Pickup` carries the kind and `PickupCount` the amount, both of them numbers the game gives meaning to. `Collector` is the reach a character gathers within, and `Taken` is the flag the crate writes so a pickup is claimed once. `Middle` is an optional lift added to a collector's position, for a body whose transform sits at its feet. The game decides what happens to a claimed pickup, whether that is despawning it, adding it to an inventory, or playing a sound.

## Driving it

```
just example pickup
```

The arrow keys walk the collector over the scattered coins. A coin within its reach is claimed and taken off the floor, and the readout counts the purse, the coins still lying and the reach that claimed them.
