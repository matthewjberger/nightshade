---
name: reference_default_entity_is_a_live_entity
description: Entity ids start at zero and the default entity is id zero, so a handle compared with the default entity to mean "not built yet" reads a live entity as missing
type: reference
---

`Entity::default()` is id zero of generation zero, and the allocator hands
out id zero to the first entity a program spawns. A resource that keeps
entity handles and tests one against the default entity to know whether the
handles were filled reads the first entity as an empty slot, and every system
behind that test returns without doing its work. Which entity comes first
depends on which startup system reaches the command queue first, so the test
holds in one app and fails in the next.

A handles resource carries a `built` flag that the builder sets, or wraps the
handle in an `Option`, and the systems that read it test that.
