---
description: Sweep the repo for the antipatterns in docs/antipatterns.md and refactor every instance away. Components that are secretly objects first.
---

# Sweep Antipatterns

Read `docs/antipatterns.md` first. It is the list, and it is the only list. Do
not invent entries and do not widen the sweep to style you happen to dislike.

Then work through the steps in order.

## Step 1: pick the ground

Ask what to sweep if the prompt did not say. Default to the apps, because they
are where the pattern breeds:

- One app, named: `apps/games/<name>/src`, `apps/features/<name>/src`.
- The apps: `apps/**/src`.
- Everything: `apps/**/src` and `crates/**/src`.

A sweep of the crates is a larger change than a sweep of one app, so say which
you are about to do before you start.

## Step 2: find the candidates

For each ground, look in `components.rs` and in any `components/` directory.
Those are the only files that declare components, so the sweep does not need to
read the whole app.

A struct in one of those files is a candidate when either holds:

- It has two or more fields.
- It has no fields at all.

Read each candidate. A candidate is a real instance when the fields travel
together only because the struct names a kind of thing, and a system would
rather read one of them. Note which systems read it and which fields each one
touches.

Two things are not instances, and leaving them alone is part of doing this
correctly:

- A type whose fields are one value that cannot be split. A transform is a
  matrix. A region is a pair of corners. Splitting those makes the code worse.
- Anything in `resources.rs`. Data shared across entities is a resource and
  keeps whatever shape it needs.

List what you found before you change anything. Say, for each instance, the
struct, its fields, and the systems that read it.

## Step 3: refactor one instance at a time

For each instance, in this order:

1. Replace the struct with one newtype for each field, in the same file. Give
   each the derives the ECS needs: `Default`, `Clone`, `Copy` where the value
   is small, and `Debug`.
2. Reuse a newtype that already exists rather than declaring a second one of
   the same value. Two kinds of entity sharing an `Angle` is the point.
3. Change every spawn to set each component on the entity.
4. Change every query to name the components the system actually touches, and
   drop the ones it does not.
5. Where two kinds of entity now match the same query, narrow it with
   `Without<T>` on a component only one of them carries. Never add a marker
   struct to tell them apart. A marker is the same antipattern with the fields
   removed.
6. Build the one crate: `cargo check -p <name>`. Fix what broke before you move
   to the next instance.

Do not batch the instances. A sweep that changes six structs at once and then
meets forty errors is slower than six small ones.

## Step 4: prove it

In order:

1. `just format`
2. `cargo clippy -p <name> --all-targets` for each crate you touched, and fix
   every warning. Remove dead code rather than suppressing it.
3. Launch each app you touched once: `just run <name> --capture <path>`. The
   refactor moves data between archetypes, so a first frame that still draws is
   the proof that the spawns and the queries still agree.

Do not run `just lint` or `just test` across the workspace. CI runs those.

## Step 5: report

Say what changed, in plain sentences: which structs became which components,
which queries narrowed, and which candidates you left alone and why. If a
sweep found nothing, say that.
