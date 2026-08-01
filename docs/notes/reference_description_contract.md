# What the description language promises, and what the interpreter may choose

This is the document a second interpreter would be written against. It names
nothing inside the one that exists. Every sentence here is one of three kinds.
A promise is something a description may rely on. A choice is something the
interpreter owns and may change. A refusal is a question the description is not
allowed to ask, and the checker turns it into a fault with a message that says
which rule it broke.

The rule behind all of it: a freedom has to be bought. Where the interpreter
wants room to move, either the answer cannot see the difference, or the
description says in writing that it gives that room up.

## A frame

**Promise.** A frame reads the world once, runs the systems, and settles. What a
system reads is the world as it stood at the start of its phase, plus whatever
that system itself has written.

**Choice.** Everything about how the work is divided, how the values are stored,
and in what order two systems of one phase run.

## The order systems run in

**Promise.** A system that reads a value runs after the system that writes it.
An order named between two systems holds. A system that provides a contract runs
before a system that needs it. Systems of one phase see the same world, and what
they write reaches the world together at the end of that phase.

**Choice.** The order of two systems in one phase that do not feed each other.

**Refusal.** Two systems in one phase that write the same value, unless both of
them only add to it.

## The order things run in inside a pick

**Promise.** A pick that names one thing names the same one every time, the one
with the lowest identity among those that match. A pick that gathers things
gathers them in rising identity. A pick that runs a body once for each thing
runs that body once for each thing, and what it writes reaches the world as if
the things had run in rising identity.

**Choice.** The order the bodies actually run in, and whether they run at the
same time as each other.

That last promise is the whole of the trick. The body of one thing cannot see
what the body of another thing wrote, because no write reaches the world until
the pick is done, and the writes are put in identity order before they land. A
sum comes out the same however the work was split, and a replay of the frame
gives the same numbers.

## Reading another thing

**Promise.** A body may read the values of a thing other than its own. It reads
them as they stood at the start of the phase.

**Choice.** Nothing here is open.

## Removing a thing

**Promise.** A removal takes effect at once, for the rest of the frame. A body
that walks a gathered set skips a thing that has been removed.

**Refusal.** A removal inside a pick that states any order. What a later thing
sees depends on which things ran first, and a pick with no stated order has no
answer to that.

## Leaving early

**Promise.** Leaving a loop over a list leaves that loop. Stopping a system
stops it, and the things the pick had not reached do not run.

**Refusal.** Stopping a system inside a pick that states any order. Which things
ran before the stop is the order itself.

## Setting a value and adding to a value

**Promise.** Setting is exclusive and adding commutes. A system sees its own
writes at once, in the order it made them, so a value written on one line and
read on the next reads back what was written. Setting a value after adding to it
throws the addition away.

**Choice.** How the writes are carried until the phase ends.

**Refusal.** Two systems in one phase that both set the same value, and a system
that sets a value that another in its phase adds to.

## Adding numbers over many things

**Promise.** The total is the same whatever order the things ran in, and the
same on a replay.

**Choice.** How the sum is reached.

Adding small numbers to a large one does not give the same answer in every
order, so the interpreter puts the additions of a pick in identity order before
it makes the sum. A description cannot see how, only that the answer holds
still.

## Events

**Promise.** A system reads the events of the phases that finished before its
own. Events arrive in the order they were announced, where the announcements of
one pick come in rising identity and the systems come in the order they ran.
An event announced inside a phase does not reach a reader in that same phase.

**Choice.** Nothing here is open.

## Where a name comes from

**Promise.** A name belongs to the module that declares it. A file that names
something looks first at what it declares itself, then at what the files it uses
declare, and takes the first it finds. Writing the module in front of a name
picks that module's one and nothing else. Two files may therefore declare the
same name and neither sees the other's.

**Choice.** How the names are kept apart. A description that stands alone is
never told about modules at all.

**Refusal.** A name that nothing declares and nothing serves.

A use may carry a block of numbers, and each one replaces a default in the
module being used. The module states what it is and the description that
composes it states what it wants.

## Asking and announcing

**Promise.** A description has two channels and they mean different things. A
command is a request. One system sends it and exactly one system handles it. An
event is an announcement. Any number of systems read it and none has to.

A declaration may say what a channel carries. When it does, every send and every
announcement of that channel names those values and no others, in the order the
declaration gives. When it does not, the values are read from the sites that
fill it.

**Choice.** How the two channels are carried. They are the same thing at run
time and only the rules differ.

**Refusal.** Sending an event, announcing a command, a command nothing sends, a
command nothing handles, a command two systems handle, a value a declaration
does not name, and a value a declaration names that a site leaves out.

## The draw

**Promise.** A draw gives a number between zero and one. The same frame drawn
again gives the same numbers. The number a thing draws depends on that thing,
on how many times it has drawn this frame, and on the frame, and on nothing
else.

**Choice.** Which numbers. A description may not rely on a particular one, and
the interpreter may change how it makes them.

This is what lets a pick be split. Before, the number a thing drew depended on
how many things had drawn before it, so any change in order changed every draw
after it.

## What the frame writes

**Promise.** Four writes happen outside any system, at named points.

Before the systems run, the frame writes what it sensed: the clock, the input
and where the pointer is aiming. Before a body runs, a pick fills any value that
carries a starting number and that the thing does not have yet, and that number
is the same for every thing, because it may not read the thing. After the
systems run, the frame counts down anything with a lifetime and removes what
reaches zero. Last, the frame clears every pulse.

**Refusal.** A pure system that names what the frame sensed.

## What pure promises

**Promise.** A pure system is a function of what it declares it reads. Given the
same readings it writes the same things.

**Choice.** Whether to run it at all. An interpreter may skip a pure system when
nothing it reads has changed since the last time it ran.

**Refusal.** A pure system that draws, and a pure system that names the clock,
the input, the random block, the flow or the shake.

## Replay

**Promise.** Restoring a frame and running forward gives the same frames again.

This is the test every rule above has to pass. The tape restores a frame and
runs on from it, so anything a description can observe that the restored state
does not carry is a fault waiting for a seek. That is why the draw carries a
frame count, why the additions of a pick are ordered before they land, and why
the writes the frame makes on its own are named here rather than left to be
noticed.

## Where the rules are enforced

The checker refuses a description that breaks a refusal, and the message names
the system and the rule. A description that states no order anywhere gets every
promise above and gives up nothing, which is why none of the descriptions in
`apps/games/dsl/games/` had to change.

`reference_description_commands.md` describes how the interpreter that exists
keeps these promises.
