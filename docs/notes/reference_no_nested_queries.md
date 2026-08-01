---
name: reference_no_nested_queries
description: A query call inside another query's closure panics with a RefCell already borrowed message, so gather into a Vec first
type: reference
---

In nightshade-ecs a lazily built `Query` borrows the whole storage cell for the duration of `for_each`, `for_each_mut`, or `for_entity`. Calling a second query inside the first one's closure panics at `crates/nightshade-ecs/src/system_param.rs` with "RefCell already borrowed".

**Why:** it looks natural to write `parents.for_each(|_, (parent,)| children.for_entity(parent.child...))`, and it compiles cleanly. It only fails at run time, on the frame the outer query first matches a row.

**How to apply:** gather what the outer pass needs into a `Vec` of plain values, let the closure end, then run the second query over that vector. See `paint_the_structures` in [[project_srpg_app]] for the shape.
