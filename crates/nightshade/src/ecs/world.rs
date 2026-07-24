//! ECS world definition and component/resource re-exports.
//!
//! The [`World`] struct is the central data store for all game state, its components
//! registered through the `nightshade_ecs::dynamic_schema!` macro. It contains:
//!
//! - **Entities**: Created with `spawn_entities(world, flags, count)`, identified by [`Entity`] (u64)
//! - **Components**: Bitflag-selected data (e.g., `LOCAL_TRANSFORM | RENDER_MESH`)
//! - **Resources**: Global singletons in the ECS resource map, reached via `Res<T>` / `world.res::<T>()`
//!
//! Key resource paths:
//!
//! - `world.res::<crate::ecs::time::Time>()` â€” frame timing ([`Time`](crate::ecs::time::Time))
//! - `world.res::<crate::platform::input::resources::Input>()` â€” keyboard/mouse/touch state ([`Input`](resources::Input))
//! - `world.res::<crate::render::config::RenderSettings>()` â€” shading and post-process settings ([`RenderSettings`](crate::render::config::RenderSettings))
//! - `world.res::<crate::ecs::camera::resources::ActiveCamera>().0` â€” current camera entity
//! - `world.res::<crate::ecs::entity_registry::EntityRegistry>().names` â€” name-to-entity mapping

pub use crate::platform::input::events::AppEvent;

pub mod cleanup;
pub mod commands;
pub mod scratch;
pub use commands::{CommandQueues, EcsCommand, RenderCommand, load_hdr_skybox};
#[cfg(not(target_arch = "wasm32"))]
pub use commands::{capture_screenshot, capture_screenshot_to_path};
pub use scratch::Scratch;

pub use nalgebra_glm::{Mat4, Quat, Vec2, Vec3, Vec4};
pub use nightshade_ecs::Entity;

nightshade_ecs::dynamic_schema!(@consts 1u64;
    ANIMATION_PLAYER,
    AUDIO_LISTENER,
    AUDIO_SOURCE,
    BOUNDING_VOLUME,
    CAMERA_CULLING_MASK,
    CAMERA_ENVIRONMENT,
    CAMERA_POST_PROCESS,
    CAMERA,
    CASTS_SHADOW,
    CHARACTER_CONTROLLER,
    CLOTH,
    COLLIDER,
    COLLISION_LISTENER,
    CONSTRAINED_ASPECT,
    CULLING_MASK,
    DECAL,
    GLOBAL_TRANSFORM,
    GUID,
    HOVERED,
    IGNORE_PARENT_SCALE,
    INSTANCED_MESH,
    JOINT,
    LIGHT,
    LINES,
    LOCAL_TRANSFORM,
    MATERIAL_REF,
    MATERIAL_VARIANTS,
    MORPH_WEIGHTS,
    NAME,
    NAVMESH_AGENT,
    PAN_ORBIT_CAMERA,
    PARENT,
    PARTICLE_EMITTER,
    PHYSICS_INTERPOLATION,
    PREFAB_SOURCE,
    RENDER_LAYER,
    RENDER_MESH,
    RIGID_BODY,
    ROTATION,
    SCRIPT,
    SKIN,
    TEXT_CHARACTER_BACKGROUND_COLORS,
    TEXT_CHARACTER_COLORS,
    TEXT,
    THIRD_PERSON_CAMERA,
    VIEWPORT_SHADING,
    VIEWPORT_UPDATE_MODE,
    VISIBILITY,
    WATER,
    BEAM,
    LIGHTNING_BOLT,
    TRAIL,
    VFX_ANIMATOR,
    MESHLET_MESH
);

pub const CORE: usize = 0;
pub const UI: usize = 1;
pub const RENDER: usize = 2;

/// The renderer-facing identifier of the world whose scene is being composed,
/// handed to each frame's `RenderInputs`.
#[derive(Default)]
pub struct WorldId(pub u64);

pub use nightshade_ui::schema::*;

/// Index of an app's game member world in `world.ecs.worlds`, the first
/// slot after the engine's members. Apps register their schema there once
/// at initialize: `world.ecs.add_world_at(GAME, register_game_components())`.
pub const GAME: usize = 3;

/// The engine's ECS: a [`nightshade_ecs::dynamic::DynEcs`] group with the core and
/// retained-UI member worlds over one shared entity allocator, plus the
/// engine's resources. The group keeps the lifecycle log (handle allocation
/// and death anywhere); each member world keeps its own row-level structural
/// log. Member access is `world.ecs.worlds[CORE]` / `world.ecs.worlds[UI]`;
/// group operations (spawning, despawn broadcast, liveness) are available
/// directly on `World` through deref.
pub struct World {
    pub ecs: nightshade_ecs::dynamic::DynEcs,
}

impl World {
    /// Borrows a plugin-owned resource stored in the ECS resource map,
    /// panicking with the type name if the owning plugin was never composed.
    pub fn plugin_resource<T: Send + Sync + 'static>(&self) -> &T {
        self.ecs.resource::<T>().unwrap_or_else(|| {
            panic!(
                "plugin resource {} is missing; compose the plugin that owns it",
                std::any::type_name::<T>()
            )
        })
    }

    /// Mutably borrows a plugin-owned resource stored in the ECS resource
    /// map, panicking with the type name if the owning plugin was never
    /// composed.
    pub fn plugin_resource_mut<T: Send + Sync + 'static>(&mut self) -> &mut T {
        self.ecs.resource_mut::<T>().unwrap_or_else(|| {
            panic!(
                "plugin resource {} is missing; compose the plugin that owns it",
                std::any::type_name::<T>()
            )
        })
    }

    /// Writes `T` on the member world that registered it, exactly as the
    /// group-typed [`DynEcs::set`](nightshade_ecs::dynamic::DynEcs::set) does. A type no
    /// member world has registered is an app component: it lazily registers into
    /// the app member world ([`GAME`]), creating that world the first time one is
    /// written, so game code stores components on engine entities without
    /// declaring a schema. Reads of an unregistered type already come back empty
    /// (`get`, `has`, `query_ref`); a component a game serializes into snapshots
    /// still earns an explicit `serde` schema, which lazy registration cannot
    /// carry.
    pub fn set<T: Send + Sync + Default + 'static>(&mut self, entity: Entity, value: T) {
        let index = match self.ecs.route::<T>() {
            Some(index) => index,
            None => {
                if self.ecs.worlds.len() <= GAME {
                    self.ecs
                        .add_world_at(GAME, nightshade_ecs::dynamic::ComponentRegistry::default());
                }
                GAME
            }
        };
        self.ecs.worlds[index].set(entity, value);
    }
}

impl std::ops::Deref for World {
    type Target = nightshade_ecs::dynamic::DynEcs;

    fn deref(&self) -> &Self::Target {
        &self.ecs
    }
}

impl std::ops::DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ecs
    }
}

impl nightshade_ecs::dynamic::ResourceHost for World {
    fn resource_map_mut(&mut self) -> &mut nightshade_ecs::dynamic::ResourceMap {
        &mut self.ecs.resources
    }

    fn resource_map(&self) -> &nightshade_ecs::dynamic::ResourceMap {
        &self.ecs.resources
    }
}

impl nightshade_ecs::system_param::EventHost for World {
    fn event_bus_mut(&mut self) -> &mut nightshade_ecs::dynamic::EventBus {
        &mut self.ecs.events
    }

    fn event_bus(&self) -> &nightshade_ecs::dynamic::EventBus {
        &self.ecs.events
    }
}

pub use crate::platform::input::resources::DroppedFile;
