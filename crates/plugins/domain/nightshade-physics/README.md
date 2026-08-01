# nightshade-physics

`nightshade-physics` runs rigid body simulation over the world with rapier3d. An entity that carries a `RigidBodyComponent` and a `ColliderComponent` is stepped on a fixed timestep and its transform is written back, with interpolation between steps so motion stays smooth at any frame rate. The crate also carries character controllers, joints, shape and ray queries, and the collision events a game reacts to.

## Composing it

```rust
nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled("Coin Dash"))
    .add_plugin(nightshade_physics::PhysicsPlugin)
    .add_plugin(CoinDashPlugin)
    .run()
```

`PhysicsPlugin` inserts `PhysicsWorld` and `PickingWorld`, fills the camera collision hook with its own sweep, and registers the simulation step and the collision event emitter on `Stage::FrameUpdate`.

## Using it

```rust
commands.set(
    entity,
    RigidBodyComponent::new_static().with_translation(0.0, -0.05, 0.0),
);
commands.set(
    entity,
    ColliderComponent::new_cuboid(half_extent, 0.05, half_extent)
        .with_friction(0.8)
        .with_restitution(0.1),
);
```

Setting the two components on an entity is the whole of putting it in the simulation, and taking them off takes it out again. `RigidBodyComponent` has `new_static`, `new_dynamic` and `new_kinematic` with builder calls for the translation, the rotation, the mass, the damping and the gravity scale, while `ColliderComponent` has `new_cuboid`, `new_ball` and `new_cylinder` with friction and restitution. `CharacterControllerComponent::new_capsule` drives a player that walks rather than tumbles, and `spawn_character_body` builds one already carrying its interpolation. `PhysicsWorld` holds the gravity, the fixed timestep and the `enabled` flag a paused game clears, and `CollisionEvent` reaches a game through `EventReader`.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-physics = { path = "../nightshade-physics", default-features = false, features = ["contract"] }
```

The contract carries the rigid body, collider, character controller and interpolation components, the handle and group types they name, and the collision and contact force events, with no rapier3d pipeline behind them.
