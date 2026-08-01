# nightshade-inventory

The slots a character carries and the rules for stacking into them. `Slots` is a row of `Slot` values on an entity and `Selected` is the position of the one in hand, so a hotbar, a chest, and a shop stock are the same shape on three different entities. A kind is a plain `u32` the game assigns meaning to, which keeps the crate out of the business of what an item is.

## Composing it

This crate carries no plugin. A game names it in its own `Cargo.toml` and inserts `StackLimit` for the whole program.

```rust
app.insert_resource(StackLimit(64));
```

`StackLimit::default()` is sixty four. A caller holds the `Slots` and `Selected` components on whichever entity carries the bag and writes them back through the storage.

## Using it

```rust
use nightshade_inventory::{Selected, Slots, StackLimit};
use nightshade_inventory::commands::{add, make_room, select, select_next, swap, take};
use nightshade_inventory::queries::{held, in_hand, room_for};

let limit = resources.res::<StackLimit>();
let mut slots = Slots::default();
make_room(&mut slots, 8);

let stored = add(&mut slots, limit, kind, 3);
let removed = take(&mut slots, kind, 1);
swap(&mut slots, 0, 4);

let mut selected = Selected(0);
select(&mut selected, &slots, 3);
select_next(&mut selected, &slots);

let total = held(&slots, kind);
let slot = in_hand(&slots, &selected);
let room = room_for(&slots, kind, limit.0);

storage.set(holder, slots);
storage.set(holder, selected);
```

`make_room` sizes the row and clears it. `add` tops up the partial stacks of that kind first and then fills empty slots, and it returns how many it actually stored, so a caller can leave the remainder on the ground. `take` pulls from every slot of that kind until the count is met and clears the kind off a slot it empties, returning how many it got. `select` clamps to the last slot and `select_next` wraps, so neither can point past the row. `held` sums one kind across the whole bag, `in_hand` reads the selected slot, and `room_for` reports how many more of a kind would fit.

## Driving it

```
just example inventory
```

Press 1, 2 or 3 to put one of that kind into the bag and space to take one back out of the slot in hand. The left and right arrows move the hand along the row and X swaps the slot in hand with the one after it. Each post rises with the count in its slot, the post in hand stands wider than the rest, and the readout gives the slot in hand and the total carried of every kind.
