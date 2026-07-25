//! Physics simulation using Rapier3D.
//!
//! Provides rigid body dynamics, collision detection, and character controllers:
//!
//! - [`RigidBodyComponent`]: Dynamic, kinematic, or static physics body
//! - [`ColliderComponent`]: Collision shape with physical material properties
//! - [`ColliderShape`]: Ball, cuboid, capsule, cylinder, cone, convex mesh, trimesh, heightfield
//! - [`CharacterControllerComponent`]: First-person kinematic character with walking, jumping, crouching
//! - [`PhysicsInterpolation`]: Smooth rendering between fixed timestep physics updates
//!
//! The physics feature is optional. Component types are always available for serialization,
//! but simulation requires the `physics` feature flag to be enabled.
//!
//! # First-Person Character Controller
//!
//! The easiest way to add player movement is with the built-in first-person controller:
//!
//! ```ignore
//! // Returns the player body and its attached first-person camera.
//! let (player, camera) = spawn_first_person_player(world, Vec3::new(0.0, 2.0, 0.0));
//! world.res_mut::<crate::ecs::camera::resources::ActiveCamera>().0 = Some(camera);
//! ```
//!
//! Then run the controller system each frame:
//!
//! ```ignore
//! fn run_systems(&mut self, world: &mut World) {
//!     character_controller_system(world);
//! }
//! ```
//!
//! # Dynamic Rigid Bodies
//!
//! Create objects affected by gravity and collisions:
//!
//! ```ignore
//! let entity = spawn_cube_at(world, Vec3::new(0.0, 5.0, 0.0));
//! world.set(entity, RigidBodyComponent::new_dynamic().with_mass(10.0));
//! world.set(entity, ColliderComponent::new_cuboid(0.5, 0.5, 0.5).with_restitution(0.3));
//! ```
//!
//! The rigid body and collider components register into the physics plugin's
//! own member world; the step's initialize pass builds the Rapier bodies from
//! them on the next frame.
//!
//! # Static Colliders
//!
//! Create immovable collision geometry (floors, walls):
//!
//! ```ignore
//! let floor = spawn_plane_at(world, Vec3::zeros());
//! world.set(floor, RigidBodyComponent::new_static());
//! world.set(floor, ColliderComponent::new_cuboid(50.0, 0.1, 50.0));
//! ```
//!
//! # Trigger Volumes (Sensors)
//!
//! Create colliders that detect overlaps without physical response:
//!
//! ```ignore
//! world.set(entity, ColliderComponent::new_ball(2.0).as_sensor());
//! ```
//!
//! # Physics World Access
//!
//! For advanced queries and direct Rapier access:
//!
//! ```ignore
//! let physics = world.plugin_resource::<PhysicsWorld>();
//! // Ray casting, overlap queries, etc. via physics.rigid_body_set, physics.collider_set
//! ```
//!
//! # Fixed Timestep
//!
//! Physics runs at a fixed 60 Hz timestep with interpolation for smooth rendering.
//! Configure via `world.plugin_resource_mut::<PhysicsWorld>().fixed_timestep`.

pub mod components;
pub mod types;

#[cfg(feature = "physics")]
pub mod camera_collision;
#[cfg(feature = "physics")]
pub mod character_controller;
#[cfg(feature = "physics")]
pub mod collision_events;
#[cfg(feature = "physics")]
pub mod commands;
#[cfg(all(feature = "physics", feature = "debug_render"))]
pub mod debug;
#[cfg(feature = "physics")]
pub mod events;
#[cfg(feature = "physics")]
pub mod foot_ik;
#[cfg(feature = "physics")]
pub mod joints;
#[cfg(all(feature = "physics", feature = "scene_graph"))]
pub mod prefab;
#[cfg(feature = "physics")]
pub mod ragdoll;
#[cfg(feature = "physics")]
pub mod resources;
#[cfg(all(feature = "physics", feature = "scene_graph"))]
pub mod scene;
#[cfg(feature = "physics")]
pub mod schema;
#[cfg(feature = "physics")]
pub mod snapshot;
#[cfg(feature = "physics")]
pub mod systems;

pub use components::*;
pub use types::*;

#[cfg(feature = "physics")]
pub use character_controller::*;
#[cfg(feature = "physics")]
pub use commands::*;
#[cfg(all(feature = "physics", feature = "debug_render"))]
pub use debug::*;
#[cfg(feature = "physics")]
pub use events::*;
#[cfg(feature = "physics")]
pub use joints::*;
#[cfg(feature = "physics")]
pub use resources::*;
#[cfg(feature = "physics")]
pub use schema::physics_component_world;
#[cfg(feature = "physics")]
pub use systems::*;

/// Enables the simulation. Pause and resume through
/// `world.plugin_resource_mut::<PhysicsWorld>().enabled`.
#[cfg(feature = "physics")]
pub fn install(world: &mut crate::ecs::world::World) {
    if world.ecs.resource::<PhysicsWorld>().is_none() {
        world.ecs.insert_resource(PhysicsWorld::default());
    }
    world.plugin_resource_mut::<PhysicsWorld>().enabled = true;
}

/// Registers the physics step and collision-event systems into the frame
/// update stage, stepping from the start.
#[cfg(feature = "physics")]
pub fn register_frame_systems(stages: &mut nightshade_ecs::Stages<crate::ecs::world::World>) {
    use crate::app::{Stage, push_frame_system};
    push_frame_system(stages, Stage::FrameUpdate, systems::run_physics_systems);
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        collision_events::emit_collision_events_system,
    );
    // Runs in FramePostUpdate, after the core transform propagation, so the
    // proxy bodies (written as local transforms by the step in FrameUpdate)
    // and the skeleton bones it reads back from carry freshly propagated,
    // same-frame global transforms.
    push_frame_system(stages, Stage::FramePostUpdate, ragdoll::ragdoll_sync_system);
}

/// Installs the physics simulation: [`install`] enables it and
/// [`register_frame_systems`] adds the step. Pause and resume at runtime
/// through `world.plugin_resource_mut::<PhysicsWorld>().enabled`, the way
/// games gate physics per screen.
#[cfg(feature = "physics")]
pub struct PhysicsPlugin;

#[cfg(feature = "physics")]
impl crate::app::Plugin for PhysicsPlugin {
    fn build(&self, app: &mut crate::app::App) {
        app.world
            .insert_resource(crate::plugins::physics::resources::PhysicsWorld::default());
        install(&mut app.world);
        camera_collision::install_camera_collision(&mut app.world);
        foot_ik::install_foot_ik_ground_query(&mut app.world);
        #[cfg(feature = "scene_graph")]
        scene::register_scene_hooks(&mut app.world);
        #[cfg(feature = "scene_graph")]
        prefab::register_prefab_hooks(&mut app.world);
        snapshot::register_snapshot_capability(&mut app.world);
        register_frame_systems(&mut app.stages);
    }
}
