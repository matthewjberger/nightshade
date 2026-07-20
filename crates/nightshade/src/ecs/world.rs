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
//! - `world.res::<crate::ecs::input::resources::Input>()` â€” keyboard/mouse/touch state ([`Input`](resources::Input))
//! - `world.res::<crate::render::config::RenderSettings>()` â€” shading and post-process settings ([`RenderSettings`](crate::render::config::RenderSettings))
//! - `world.res::<crate::ecs::camera::resources::ActiveCamera>().0` â€” current camera entity
//! - `world.res::<crate::ecs::entity_registry::EntityRegistry>().names` â€” name-to-entity mapping

pub mod components {
    pub use crate::ecs::animation::components::{
        AnimationChannel, AnimationClip, AnimationInterpolation, AnimationPlayer,
        AnimationProperty, AnimationSampler, AnimationSamplerOutput, AnimationValue,
    };
    pub use crate::ecs::camera::components::{
        Camera, CameraEnvironment, CameraPostProcess, ConstrainedAspect, FogOverride,
        OrthographicCamera, PanOrbitCamera, PerspectiveCamera, Projection, ShadingMode, Smoothing,
        ViewportShading,
    };
    pub use crate::plugins::audio::components::{AudioBus, AudioListener, AudioSource};
    pub use crate::render::bounding_volume::{BoundingVolume, OrientedBoundingBox};
    pub use crate::render::config::ViewportUpdateMode;

    pub use crate::ecs::cloth::components::{Cloth, ClothPinning};
    pub use crate::ecs::decal::components::Decal;
    pub use crate::ecs::input::components::Hovered;
    pub use crate::ecs::light::components::{AreaLightShape, Light, LightType};
    pub use crate::ecs::lines::components::{Line, Lines, MAX_LINES};
    pub use crate::ecs::material::components::{
        MaterialRef, MaterialVariantMapping, MaterialVariants,
    };
    pub use crate::ecs::mesh::components::{InstanceTransform, InstancedMesh, RenderMesh};
    pub use crate::ecs::meshlet::components::MeshletMesh;
    pub use crate::ecs::morph::components::MorphWeights;
    pub use crate::ecs::primitives::CastsShadow;
    pub use crate::ecs::primitives::EasingFunction;
    pub use crate::ecs::primitives::Guid;
    pub use crate::ecs::primitives::Name;
    pub use crate::ecs::primitives::Visibility;
    pub use crate::ecs::primitives::{CameraCullingMask, CullingMask};
    pub use crate::ecs::skin::components::{Joint, Skin};
    pub use crate::ecs::text::components::{
        Text, TextCharacterBackgroundColors, TextCharacterColors, TextProperties,
    };
    pub use crate::ecs::transform::components::{
        ChildOf, GlobalTransform, LocalTransform, Rotation,
    };
    pub use crate::ecs::ui::components::{
        StateTransition, UiButtonData, UiDepthMode, UiLayoutNode, UiLayoutRoot, UiNodeColor,
        UiNodeContent, UiNodeInteraction, UiStateWeights, UiThemeBinding,
    };
    pub use crate::ecs::vfx::components::{Beam, LightningBolt, Trail, VfxAnimator, VfxHandle};
    pub use crate::ecs::water::components::Water;
    pub use crate::plugins::physics::components::{
        CharacterControllerComponent, ColliderComponent, ColliderShape, CollisionListener,
        PhysicsInterpolation, RigidBodyComponent,
    };
    pub use crate::render::material::{AlphaMode, Material};
    pub use crate::render::particles::{
        ColorGradient, EmitterShape, EmitterType, ParticleEmitter, ParticleSystemStats,
        ParticleTextureUpload,
    };
    pub use crate::render::render_layer::RenderLayer;
    pub use crate::render::text_data::{TextAlignment, VerticalAlignment};
}

pub mod resources {
    pub use crate::ecs::asset_state::AssetState;
    pub use crate::ecs::camera::resources::ActiveCamera;
    pub use crate::ecs::cloth::resources::ClothMeshSync;
    pub use crate::ecs::entity_registry::EntityRegistry;
    pub use crate::ecs::event::Events;
    pub use crate::ecs::event_bus::resources::{EventBus, Message};
    pub use crate::ecs::graphics::selection::Selection;
    pub use crate::ecs::input::resources::{Input, MouseState};
    pub use crate::ecs::loading::LoadingState;
    pub use crate::ecs::material::resources::MaterialRegistry;
    #[cfg(feature = "meshlet")]
    pub use crate::ecs::meshlet::resources::MeshletAssets;
    #[cfg(feature = "assets")]
    pub use crate::ecs::prefab::resources::CachedPrefab;
    pub use crate::ecs::prefab::resources::{AnimationCache, PrefabCache};
    pub use crate::ecs::text::resources::{TextCache, TextState};
    pub use crate::ecs::time::Time;
    pub use crate::ecs::transform::resources::TransformState;
    pub use crate::ecs::ui::UserInterface;
    pub use crate::ecs::ui::resources::RetainedUiState;
    pub use crate::ecs::window::resources::Window;
    pub use crate::ecs::world::WorldId;
    pub use crate::ecs::world::cleanup::CleanupState;
    #[cfg(feature = "audio")]
    pub use crate::plugins::audio::resources::AudioEngine;
    #[cfg(feature = "physics")]
    pub use crate::plugins::physics::resources::PhysicsWorld;
    pub use crate::render::config::ViewportRect;
    pub use crate::render::config::{
        Atmosphere, ColorGrading, ColorGradingPreset, DebugDraw, Fog, GpuBackend, GpuDeviceType,
        GpuProfile, MeshLodChain, MeshLodLevel, RenderSettings, RendererState, TonemapAlgorithm,
        VertexSnap,
    };
    pub use crate::render::mesh_cache::MeshCache;
    pub use crate::render::mesh_state::MeshRenderState;
    pub use crate::render::wgpu::texture_cache::TextureCache;
    pub use crate::render::wind::Wind;
    pub use nightshade_ecs::dynamic::HierarchyIndex;
}

pub mod events {
    pub use crate::ecs::event_bus::resources::{InputEvent, KeyState, Message};
    pub use crate::ecs::input::events::AppEvent;
}

pub use crate::ecs::input::events::AppEvent;

pub mod cleanup;
pub mod commands;
pub mod scratch;
pub use commands::{
    CommandQueues, EcsCommand, RenderCommand, cleanup_unused_resources_system,
    despawn_entities_with_cache_cleanup, load_hdr_skybox, load_procedural_textures,
    process_commands_system, set_material_with_textures, spawn_3d_billboard_text_with_properties,
    spawn_3d_text_with_properties, spawn_cone_at, spawn_cube_at, spawn_cylinder_at,
    spawn_instanced_mesh, spawn_instanced_mesh_with_material, spawn_mesh_at, spawn_plane_at,
    spawn_sphere_at, spawn_sun, spawn_torus_at,
};
#[cfg(not(target_arch = "wasm32"))]
pub use commands::{capture_screenshot, capture_screenshot_to_path};
pub use scratch::Scratch;

pub mod animation_systems {
    pub use crate::ecs::animation::systems::{apply_animations, update_animation_players};
}

pub use nalgebra_glm::{Mat4, Quat, Vec2, Vec3, Vec4};
pub use nightshade_ecs::Entity;

nightshade_ecs::dynamic_schema! {
    serde pub(crate) fn register_core_components {
            animation_player: crate::ecs::animation::components::AnimationPlayer => ANIMATION_PLAYER,
            audio_listener: crate::plugins::audio::components::AudioListener => AUDIO_LISTENER,
            audio_source: crate::plugins::audio::components::AudioSource => AUDIO_SOURCE,
            bounding_volume: crate::render::bounding_volume::BoundingVolume => BOUNDING_VOLUME,
            camera_culling_mask: crate::ecs::primitives::CameraCullingMask => CAMERA_CULLING_MASK,
            camera_environment: crate::ecs::camera::components::CameraEnvironment => CAMERA_ENVIRONMENT,
            camera_post_process: crate::ecs::camera::components::CameraPostProcess => CAMERA_POST_PROCESS,
            camera: crate::ecs::camera::components::Camera => CAMERA,
            casts_shadow: crate::ecs::primitives::CastsShadow => CASTS_SHADOW,
            character_controller: crate::plugins::physics::components::CharacterControllerComponent => CHARACTER_CONTROLLER,
            cloth: crate::ecs::cloth::components::Cloth => CLOTH,
            collider: crate::plugins::physics::components::ColliderComponent => COLLIDER,
            collision_listener: crate::plugins::physics::components::CollisionListener => COLLISION_LISTENER,
            constrained_aspect: crate::ecs::camera::components::ConstrainedAspect => CONSTRAINED_ASPECT,
            culling_mask: crate::ecs::primitives::CullingMask => CULLING_MASK,
            decal: crate::ecs::decal::components::Decal => DECAL,
            global_transform: crate::ecs::transform::components::GlobalTransform => GLOBAL_TRANSFORM,
            guid: crate::ecs::primitives::Guid => GUID,
            hovered: crate::ecs::input::components::Hovered => HOVERED,
            ignore_parent_scale: crate::ecs::transform::components::IgnoreParentScale => IGNORE_PARENT_SCALE,
            instanced_mesh: crate::ecs::mesh::components::InstancedMesh => INSTANCED_MESH,
            joint: crate::ecs::skin::components::Joint => JOINT,
            light: crate::ecs::light::components::Light => LIGHT,
            lines: crate::ecs::lines::components::Lines => LINES,
            local_transform: crate::ecs::transform::components::LocalTransform => LOCAL_TRANSFORM,
            material_ref: crate::ecs::material::components::MaterialRef => MATERIAL_REF,
            material_variants: crate::ecs::material::components::MaterialVariants => MATERIAL_VARIANTS,
            morph_weights: crate::ecs::morph::components::MorphWeights => MORPH_WEIGHTS,
            name: crate::ecs::primitives::Name => NAME,
            navmesh_agent: crate::plugins::navmesh::components::NavMeshAgent => NAVMESH_AGENT,
            pan_orbit_camera: crate::ecs::camera::components::PanOrbitCamera => PAN_ORBIT_CAMERA,
            child_of: nightshade_ecs::dynamic::ChildOf => PARENT,
            particle_emitter: crate::render::particles::ParticleEmitter => PARTICLE_EMITTER,
            physics_interpolation: crate::plugins::physics::components::PhysicsInterpolation => PHYSICS_INTERPOLATION,
            prefab_source: crate::ecs::prefab::components::PrefabSource => PREFAB_SOURCE,
            render_layer: crate::render::render_layer::RenderLayer => RENDER_LAYER,
            render_mesh: crate::ecs::mesh::components::RenderMesh => RENDER_MESH,
            rigid_body: crate::plugins::physics::components::RigidBodyComponent => RIGID_BODY,
            rotation: crate::ecs::transform::components::Rotation => ROTATION,
            script: crate::ecs::script::components::Script => SCRIPT,
            skin: crate::ecs::skin::components::Skin => SKIN,
            text_character_background_colors: crate::ecs::text::components::TextCharacterBackgroundColors => TEXT_CHARACTER_BACKGROUND_COLORS,
            text_character_colors: crate::ecs::text::components::TextCharacterColors => TEXT_CHARACTER_COLORS,
            text: crate::ecs::text::components::Text => TEXT,
            third_person_camera: crate::ecs::camera::components::ThirdPersonCamera => THIRD_PERSON_CAMERA,
            viewport_shading: crate::ecs::camera::components::ViewportShading => VIEWPORT_SHADING,
            viewport_update_mode: crate::render::config::ViewportUpdateMode => VIEWPORT_UPDATE_MODE,
            visibility: crate::ecs::primitives::Visibility => VISIBILITY,
            water: crate::ecs::water::components::Water => WATER,
            beam: crate::ecs::vfx::components::Beam => BEAM,
            lightning_bolt: crate::ecs::vfx::components::LightningBolt => LIGHTNING_BOLT,
            trail: crate::ecs::vfx::components::Trail => TRAIL,
            vfx_animator: crate::ecs::vfx::components::VfxAnimator => VFX_ANIMATOR,
            // Registered whether or not the `meshlet` feature is on. The schema
            // macro takes no attribute on a field, and cfg-ing one out would be
            // worse than the leak: the bit constants are assigned textually
            // while the masks are assigned by registration order, so dropping a
            // registration shifts every later component's mask out from under
            // its constant. The component is a bare id and the systems, assets,
            // and pass behind it are all gated, so with the feature off this
            // costs one mask bit and does nothing.
            meshlet_mesh: crate::ecs::meshlet::components::MeshletMesh => MESHLET_MESH,
    }
}

nightshade_ecs::impl_component!(
    crate::ecs::animation::components::AnimationPlayer,
    crate::plugins::audio::components::AudioListener,
    crate::plugins::audio::components::AudioSource,
    crate::ecs::primitives::CameraCullingMask,
    crate::ecs::camera::components::CameraEnvironment,
    crate::ecs::camera::components::CameraPostProcess,
    crate::ecs::camera::components::Camera,
    crate::ecs::primitives::CastsShadow,
    crate::plugins::physics::components::CharacterControllerComponent,
    crate::ecs::cloth::components::Cloth,
    crate::plugins::physics::components::ColliderComponent,
    crate::plugins::physics::components::CollisionListener,
    crate::ecs::camera::components::ConstrainedAspect,
    crate::ecs::primitives::CullingMask,
    crate::ecs::decal::components::Decal,
    crate::ecs::transform::components::GlobalTransform,
    crate::ecs::primitives::Guid,
    crate::ecs::input::components::Hovered,
    crate::ecs::transform::components::IgnoreParentScale,
    crate::ecs::mesh::components::InstancedMesh,
    crate::ecs::skin::components::Joint,
    crate::ecs::light::components::Light,
    crate::ecs::lines::components::Lines,
    crate::ecs::transform::components::LocalTransform,
    crate::ecs::material::components::MaterialRef,
    crate::ecs::material::components::MaterialVariants,
    crate::ecs::morph::components::MorphWeights,
    crate::ecs::primitives::Name,
    crate::plugins::navmesh::components::NavMeshAgent,
    crate::ecs::camera::components::PanOrbitCamera,
    crate::plugins::physics::components::PhysicsInterpolation,
    crate::ecs::prefab::components::PrefabSource,
    crate::ecs::mesh::components::RenderMesh,
    crate::plugins::physics::components::RigidBodyComponent,
    crate::ecs::transform::components::Rotation,
    crate::ecs::script::components::Script,
    crate::ecs::skin::components::Skin,
    crate::ecs::text::components::TextCharacterBackgroundColors,
    crate::ecs::text::components::TextCharacterColors,
    crate::ecs::text::components::Text,
    crate::ecs::camera::components::ThirdPersonCamera,
    crate::ecs::camera::components::ViewportShading,
    crate::ecs::primitives::Visibility,
    crate::ecs::water::components::Water,
    crate::ecs::vfx::components::Beam,
    crate::ecs::vfx::components::LightningBolt,
    crate::ecs::vfx::components::Trail,
    crate::ecs::vfx::components::VfxAnimator,
    crate::ecs::meshlet::components::MeshletMesh,
);

/// The renderer-facing identifier of the world whose scene is being composed,
/// handed to each frame's `RenderInputs`.
#[derive(Default)]
pub struct WorldId(pub u64);

use crate::render::render_world::register_render_components;

pub use nightshade_ui::schema::*;

nightshade_ecs::dynamic_worlds! {
    pub fn build_engine_ecs {
        CORE => register_core_components,
        UI => register_ui_components,
        RENDER => register_render_components,
    }
}

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

impl Default for World {
    fn default() -> Self {
        #[cfg(not(any(
            feature = "physics",
            feature = "audio",
            feature = "navmesh",
            feature = "terrain",
            feature = "egui",
            all(feature = "file_watcher", not(target_arch = "wasm32")),
        )))]
        let ecs = build_engine_ecs();
        #[cfg(any(
            feature = "physics",
            feature = "audio",
            feature = "navmesh",
            feature = "terrain",
            feature = "egui",
            all(feature = "file_watcher", not(target_arch = "wasm32")),
        ))]
        let ecs = {
            let mut ecs = build_engine_ecs();
            #[cfg(feature = "physics")]
            ecs.insert_resource(crate::ecs::picking::resources::PickingWorld::default());
            #[cfg(feature = "physics")]
            ecs.insert_resource(crate::plugins::physics::resources::PhysicsWorld::default());
            #[cfg(feature = "navmesh")]
            ecs.insert_resource(crate::plugins::navmesh::resources::NavMeshWorld::default());
            #[cfg(feature = "terrain")]
            ecs.insert_resource(crate::render::terrain_config::TerrainSettings::default());
            #[cfg(feature = "egui")]
            ecs.insert_resource(crate::plugins::egui::EguiState::default());
            #[cfg(feature = "audio")]
            ecs.insert_resource(crate::plugins::audio::resources::AudioEngine::default());
            #[cfg(all(feature = "file_watcher", not(target_arch = "wasm32")))]
            ecs.insert_resource(crate::plugins::file_watcher::FileWatcher::default());
            #[cfg(all(
                feature = "assets",
                feature = "file_watcher",
                not(target_arch = "wasm32")
            ))]
            ecs.insert_resource(crate::ecs::asset_watcher::AssetWatcher::default());
            ecs
        };
        let mut ecs = ecs;
        ecs.insert_resource(crate::ecs::ui::resources::RetainedUiState::default());
        ecs.insert_resource(crate::ecs::text::resources::TextState::default());
        ecs.insert_resource(crate::ecs::ui::UserInterface::default());
        #[cfg(feature = "gizmos")]
        ecs.insert_resource(crate::plugins::gizmos::Gizmos::default());
        ecs.insert_resource(crate::ecs::entity_registry::EntityRegistry::default());
        ecs.insert_resource(crate::render::mesh_state::MeshRenderState::default());
        ecs.insert_resource(crate::ecs::mesh::components::InstancedMeshMatrices::default());
        ecs.insert_resource(nightshade_ecs::dynamic::HierarchyIndex::default());
        ecs.insert_resource(crate::schedule::RetainedUiSchedule::default());
        ecs.insert_resource(crate::render::wgpu::texture_cache::TextureCache::default());
        ecs.insert_resource(crate::ecs::world::commands::CommandQueues::default());
        ecs.insert_resource(crate::ecs::event_bus::resources::EventBus::default());
        ecs.insert_resource(crate::ecs::event::Events::default());
        ecs.insert_resource(crate::ecs::world::cleanup::CleanupState::default());
        ecs.insert_resource(crate::ecs::world::WorldId::default());
        ecs.insert_resource(crate::ecs::asset_state::AssetState::default());
        ecs.insert_resource(crate::render::config::RendererState::default());
        ecs.insert_resource(crate::ecs::camera::resources::ActiveCamera::default());
        #[cfg(feature = "grass")]
        ecs.insert_resource(crate::render::grass_config::GrassSettings::default());
        #[cfg(feature = "terrain")]
        ecs.insert_resource(crate::render::terrain_config::TerrainRenderState::default());
        ecs.insert_resource(crate::render::wind::Wind::default());
        ecs.insert_resource(crate::ecs::cloth::resources::ClothMeshSync::default());
        #[cfg(feature = "meshlet")]
        ecs.insert_resource(crate::ecs::meshlet::resources::MeshletAssets::default());
        ecs.insert_resource(crate::ecs::gpu_picking::GpuPicking::default());
        ecs.insert_resource(crate::ecs::window::resources::Monitors::default());
        ecs.insert_resource(crate::ecs::tween::Tweens::default());
        ecs.insert_resource(crate::ecs::script::components::GlobalScripts::default());
        ecs.insert_resource(crate::ecs::graphics::selection::Selection::default());
        ecs.insert_resource(crate::render::config::DebugDraw::default());
        ecs.insert_resource(crate::ecs::cutscene::resources::CutsceneDirector::default());
        ecs.insert_resource(crate::ecs::transform::resources::TransformState::default());
        ecs.insert_resource(crate::ecs::graphics::scene_sync::RenderSceneSync::default());
        ecs.insert_resource(crate::ecs::world::scratch::Scratch::default());
        ecs.insert_resource(crate::ecs::ui::resources::UiImageLayerAllocator::default());
        ecs.insert_resource(crate::ecs::scene::typed::ComponentTypeRegistry::default());
        ecs.insert_resource(crate::ecs::input::resources::Input::default());
        ecs.insert_resource(crate::ecs::input::resources::Gamepad::default());
        ecs.insert_resource(crate::render::config::RenderSettings::default());
        ecs.insert_resource(crate::ecs::window::resources::Window::default());
        ecs.insert_resource(crate::render_driver::Viewport::default());
        ecs.insert_resource(crate::ecs::time::Time::default());
        ecs.insert_resource(crate::ecs::loading::LoadingState::default());
        ecs.insert_resource(crate::state::PendingState::default());
        #[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
        ecs.insert_resource(crate::plugins::steam::SteamResources::default());
        Self { ecs }
    }
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

pub use crate::ecs::input::resources::DroppedFile;
