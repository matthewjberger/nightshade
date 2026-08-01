---
name: reference_parallel_cargo_builds_oom
description: A second parallel build exhausts this machine and leaves artifacts that read as corruption, so rerun alone with -j 6
type: reference
---

A workspace build of this size takes more memory than one machine has when a
second build is already running. `rustc` dies with `STATUS_STACK_BUFFER_OVERRUN`
or `rust_oom`, and it leaves artifacts behind that read as corruption on the
next build: `E0463 can't find crate`, `rlib format not found`, `E0786 invalid
metadata`.

Those errors mean the build ran out of memory, not that the tree is broken. Run
the build again on its own with `-j 6`. Never reach for `cargo clean`: it throws
away hours of work to fix something a rerun fixes.

If the disk fills, `target/debug/incremental` is the safe first delete, because
it rebuilds on demand.
