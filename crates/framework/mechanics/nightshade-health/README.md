# nightshade-health

Damage reaches a body as a `PendingDamage` component and the mechanic resolves it once a frame. Armor comes off the amount first, a shield soaks what it can, and the rest comes off `Health`. Whatever happens is announced as a `Hurt` event, and a body that crosses from alive to fallen sends a `Died` event carrying its `Bounty`, so a game scores a kill without reading anyone's health.

## Composing it

```rust
use nightshade_health::HealthPlugin;

app.add_plugin(HealthPlugin);
```

`HealthPlugin` registers `take_the_damage` on `Stage::LateUpdate`, which is the one writer of `Health`, `Shield` and `PendingDamage`. It announces `Hurt` and `Died` and installs no resource.

## Using it

```rust
use nightshade_health::queries::share;
use nightshade_health::{Armor, Bounty, Died, Health, MaxHealth, PendingDamage, Shield};

storage.set(body, Health(20.0));
storage.set(body, MaxHealth(20.0));
storage.set(body, Shield(0.0));
storage.set(body, Armor(2.0));
storage.set(body, PendingDamage(0.0));
storage.set(body, Bounty(1));

pub fn count_the_fallen(fallen: EventReader<Died>, bodies: Query<(&Health, &MaxHealth)>) {
    for word in fallen.iter() {
        let _ = word.bounty;
    }
    bodies.for_each(|_entity, (health, ceiling)| {
        let left = share(health, ceiling);
    });
}
```

A body needs all six components, because the mechanic's query names all six. A game deals damage by adding to `PendingDamage`, either directly or through `commands::hurt`, and the mechanic zeroes it in the same frame. `queries::share` gives the fraction a health bar draws, `fallen` reports whether a body is down, and `commands::heal` raises health with a ceiling.

## Driving it

```
just example health
```

The left and right arrows pick a body out of the row and it steps forward, Space deals a blow into its `PendingDamage`, F heals it, and R brings the whole row back to full. Each body scales to the share of health the mechanic left it, and the readout gives the chosen body's health, shield and armor beside the fallen count and the bounty the `Died` events carried.
