---
name: reference_a_parallel_batch_never_writes_a_storage_cache
description: A system on a rayon thread must find every storage cache it reads already filled, because a miss inserts into a map that its batch mates share and the heap corrupts, so a query built for a batch reads the caches and never writes them, warm fills both caches for every scheduled system, and no query leans on a scratch buffer the storage owns
type: reference
---

A parallel batch hands every system the same `Storage` through an
`UnsafeCell`, and the access model keeps them apart only on the component
columns, the resources and the event channels each one names. Anything else
on the storage that a query touches is shared by every system in the batch,
and a write there from two threads is a data race that the Windows heap
reports as `STATUS_HEAP_CORRUPTION` about a second and a half after start,
with no panic and no backtrace. Every app showed it, only sometimes, and only
at startup, because that is when the caches are cold.

Two such writes existed. `for_each` on its fast path asked
`archetype_planned_tables` for the tables of its mask, and that call inserts
into `query_plans` on a miss. `warm` filled only `query_cache`, so the first
`for_each` of a system on a worker thread was the miss, and a new table
clears `query_plans` again, which at startup happens every frame. The second
was `added_scratch`, one `Vec<bool>` on the storage that every query with an
`Added` filter cleared and resized from its own thread.

The rule now sits in the builder. A `QueryBuilder` carries `fills_caches`,
true for a query built on the storage directly, and `in_a_batch` sets it
false. The `Query` parameter and every query a `query_bundle!` builds go
through `in_a_batch`, so on a worker thread `for_each` reads the caches
through `archetype_read_cached_tables` and `archetype_read_planned_tables`,
which compute a local list on a miss and insert nothing. A query built on the
storage still fills, because whoever holds the storage holds it alone.
`warm` fills both caches, and the schedule warms a lone system as well as a
shared batch, so a scheduled system never pays for the local list. The
scratch is a local in `for_each`.

For anything new on the query path: a worker thread reads the storage's
caches and never writes them, so whatever a query needs cached is filled in
`warm` and read through the `archetype_read_` pair, and whatever it needs as
scratch it owns. The tests `a_query_built_for_a_batch_never_writes_a_storage_cache`
and `a_schedule_warms_every_system_before_it_runs` hold the two halves.
