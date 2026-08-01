# nightshade-cooldown

The wait an ability keeps before it can be used again. An entity carries how long its wait is and how much of that wait is left, and the crate spends the remainder down every frame. A game asks whether the entity is ready, uses the ability, and starts the wait over.

## Composing it

```rust
use nightshade_cooldown::CooldownPlugin;

app.add_plugin(CooldownPlugin);
```

The plugin registers `spend_the_cooldowns` on `Stage::First`, so every `Cooldown` has already been spent down by the time a game decides anything this frame. The plugin inserts no resource, because the wait lives on the entity.

## Using it

```rust
use nightshade_cooldown::commands::{clear, start, start_for};
use nightshade_cooldown::queries::{ready, share};
use nightshade_cooldown::{Cooldown, Recharge};

storage.set(gun, Recharge(1.5));
storage.set(gun, Cooldown(0.0));

if ready(&cooldown) {
    start(&mut cooldown, &recharge);
}

let left = share(&cooldown, &recharge);
start_for(&mut cooldown, 0.25);
clear(&mut cooldown);
```

`Recharge` is the full wait in seconds and `Cooldown` is what is left of it. `ready` answers whether the remainder has reached zero, and `share` gives the remainder as a fraction of the full wait, which is what a meter draws from. `start` refills the wait from the entity's own `Recharge`, `start_for` sets a wait in seconds without touching it, and `clear` makes the ability ready at once.

## Driving it

```
just example cooldown
```

Press 1 through 5 to fire that gun, space to fire every gun that is ready, R to clear all five at once and F to put a quarter second wait on all of them. Each gun stands at the height `share` reports, so it jumps up when it fires and sinks as the crate spends the wait down, and the readout gives how many are still waiting and the longest remainder.
