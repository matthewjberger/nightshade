---
name: reference_system_params_queries_last
description: A parameter system must list Res, ResMut, Commands, and other extract params first and every Query last, or the IntoSystem impl does not match and the error names only SystemCollection
type: reference
---

The `IntoSystem` impls in `crates/nightshade-ecs/src/system_param.rs` are generated as `(extract params..., queries...)`. A function whose `Query` comes before `Commands` or a `Res` is not a system, and the compiler reports only "the trait bound `fn(...) {name}: SystemCollection<_>` is not satisfied" at the `add_system` call, with no hint about the order.

**How to apply:** when that error names a plain function, reorder its parameters so every `Query<...>` sits after the last `Commands`, `Res`, `ResMut`, `EventReader`, `TextureLoad`, or `UiBuild`. Keep the parameter count modest as well, and split a wide system in two rather than adding a thirteenth parameter.
