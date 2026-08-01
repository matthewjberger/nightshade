# A system in the description language asks, and the phase boundary answers

A unit of a description never writes the store. It writes a shadow, and the
shadow becomes a list of asks when the unit ends. `commands::apply` drains that
list at a phase boundary, and nothing else changes a column, a field or an
event list.

`Machine.shadow` holds the shadow, `Store.asks` holds the queue, and
`data/ask.rs` holds the five asks: set a column, add to a column, set a field,
add to a field, and emit.

## A unit sees its own writes and nobody else's

`peek` and `peek_field` read the shadow first and the store second, so a
substep loop that writes a transform and reads it back on the next line gets
what it wrote. `breakout.ns` bounces the ball that way. Without the shadow every
substep would read the value from the start of the frame and the bounce would
never fire.

`mark` and `mark_field` fold a write into the shadow. A `:=` makes the entry
absolute and discards any delta under it. A `+=` folds into whatever is there
and keeps its kind, so an absolute entry stays absolute.

## Two writes share a value only when both commute

`:=` is exclusive and `+=` commutes. `compile::clash` refuses two units in one
phase that write the same name when either of them uses `:=`. The names come
from `Unit.sets` and `Unit.adds`, which the compiler fills as it lowers each
place.

## A phase is a group, and the default group holds one unit

`stage` gives every unit without a `phase` port a mark of its own, and every
unit that names a phase the mark of that name. It then sorts the turn order
stably by the first position of each mark, so the members of a phase sit
together. `Plan.phases` carries the mark of each seat, and `run::tick` applies
the asks when the mark of the next seat differs.

A description that names no phase therefore keeps the behavior it had when a
write reached the store immediately, because its boundary falls after every
unit. A description that names one buys the guarantee that its members read the
same snapshot, and pays a frame of delay to anything that reads a sibling.

## An indexed write copies the bank once for each unit

`Op::PokeField`, `Op::PokeColumn` and `Op::PokeColumnOf` take the index and the
value off the stack. `own_field` and `own_column` copy the list into the shadow
the first time a unit writes it and mutate that copy in place afterward, so a
loop of ten thousand writes copies the bank once. `Op::SetIndex` stays for a
list held in a local, where a copy is what the language means.

## What a contract port does

`needs` and `provides` name a contract rather than a file. `compile::served`
refuses a need that nothing provides, and `plan::order` puts a provider before
the unit that needs it. Neither port names a column, so a module can be replaced
by another that provides the same contract.

## What this costs

An emit reaches its reader at the phase boundary rather than at the statement,
so a producer and a consumer in one phase do not see each other within a frame.
`Store.asks` holds kept text and list slots, and a frame that ends without an
apply leaks those slots until the next `restore`.
