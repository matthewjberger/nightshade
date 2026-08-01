---
name: reference_check_each_app_alone
description: A workspace build unifies features and hides per-app faults, so check each app on its own
type: reference
---

`cargo check --workspace` unifies features across every member, so an app that
takes a plugin crate with `default-features = false, features = ["contract"]`
and then names its plugin type still builds, as long as some other member
enables that crate's `plugin` feature. Building the app alone fails.

The shape to look for: an app that names a plugin type, such as `CameraPlugin`,
while its own dependency line takes that crate as a contract only.

`just lint-apps-isolated` runs `cargo check -p <app>` for every app and the
template. It is chained into `just lint`.

A clean build still misses a startup panic from a resource nothing inserts.
Launch each app under a timeout and grep the log for `panicked`. A plugin that
mutates another crate's resource in its own `build` panics when composed first.
`World::resource_or_insert_default::<T>` is the fix for that shape:
`nightshade-camera` takes `RenderViews` that way and
`nightshade-camera-controller` takes `CameraCollisionHook`.

See [[reference_cross_crate_feature_forwarding_dies_on_a_merge]] for the related
failure where a feature survives a crate move but its code does not.
