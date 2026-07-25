//! Engine ECS bootstrap: the component/resource/event convenience shelves,
//! the core component-schema registration, and construction of the engine
//! `DynEcs`. Everything here names every domain, so it sits above them; the
//! substrate they depend on is `crate::ecs::world`.

use crate::ecs::world::{CORE, UI, World};
use crate::prelude::*;

pub mod components {
    pub use crate::ecs::animation::components::{
        AnimationChannel, AnimationClip, AnimationInterpolation, AnimationPlayer,
        AnimationProperty, AnimationSampler, AnimationSamplerOutput, AnimationValue,
    };
    pub use crate::ecs::camera::components::{
        Camera, CameraEnvironment, CameraPostProcess, ConstrainedAspect, FogOverride,
        OrthographicCamera, PanOrbitCamera, PerspectiveCamera, Projection, ProjectionOverride,
        RenderTarget, RenderTargetOwnership, ShadingMode, Smoothing, ViewportShading,
    };
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
    pub use crate::ecs::vfx::components::{Beam, LightningBolt, Trail, VfxAnimator, VfxHandle};
    pub use crate::ecs::water::components::Water;
    pub use crate::render::material::{AlphaMode, Material};
    pub use crate::render::particles::{
        ColorGradient, EmitterShape, EmitterType, ParticleEmitter, ParticleSystemStats,
        ParticleTextureUpload,
    };
    pub use crate::render::render_layer::RenderLayer;
    pub use crate::ui::components::{
        StateTransition, UiButtonData, UiDepthMode, UiLayoutNode, UiLayoutRoot, UiNodeColor,
        UiNodeContent, UiNodeInteraction, UiStateWeights, UiThemeBinding,
    };
    pub use nightshade_text::text_data::{TextAlignment, VerticalAlignment};
}

pub mod resources {
    pub use crate::ecs::camera::resources::ActiveCamera;
    pub use crate::ecs::cloth::resources::ClothMeshSync;
    pub use crate::ecs::entity_registry::EntityRegistry;
    pub use crate::ecs::event::Events;
    pub use crate::ecs::event_bus::resources::{EventBus, Message};
    pub use crate::ecs::graphics::selection::Selection;
    pub use crate::ecs::loading::LoadingState;
    pub use crate::ecs::material::resources::MaterialRegistry;
    #[cfg(feature = "meshlet")]
    pub use crate::ecs::meshlet::resources::MeshletAssets;
    #[cfg(feature = "assets")]
    pub use crate::ecs::prefab::resources::CachedPrefab;
    pub use crate::ecs::prefab::resources::{AnimationCache, PrefabCache};
    #[cfg(feature = "scene_graph")]
    pub use crate::ecs::scene::ScenePrefabs;
    #[cfg(feature = "scene_graph")]
    pub use crate::ecs::scene::registry::AssetRegistry;
    pub use crate::ecs::text::resources::{TextCache, TextState};
    pub use crate::ecs::texture_source::TextureSources;
    pub use crate::ecs::time::Time;
    pub use crate::ecs::transform::resources::TransformState;
    pub use crate::ecs::world::WorldId;
    pub use crate::ecs::world::cleanup::CleanupState;
    pub use crate::platform::input::resources::{Input, MouseState};
    pub use crate::platform::window::Window;
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
    pub use crate::user_interface::UserInterface;
    pub use nightshade_ecs::dynamic::HierarchyIndex;
}

pub mod events {
    pub use crate::ecs::event_bus::resources::{InputEvent, KeyState, Message};
    pub use crate::platform::input::events::AppEvent;
}

pub(crate) fn register_core_components() -> nightshade_ecs::dynamic::ComponentRegistry {
    let mut registry = nightshade_ecs::dynamic::ComponentRegistry::new();
    let key = registry.register_serde::<crate::ecs::animation::components::AnimationPlayer>();
    debug_assert_eq!(key.mask, crate::ecs::world::ANIMATION_PLAYER);
    let key = registry.register_serde::<crate::render::bounding_volume::BoundingVolume>();
    debug_assert_eq!(key.mask, crate::ecs::world::BOUNDING_VOLUME);
    let key = registry.register_serde::<crate::ecs::primitives::CameraCullingMask>();
    debug_assert_eq!(key.mask, crate::ecs::world::CAMERA_CULLING_MASK);
    let key = registry.register_serde::<crate::ecs::camera::components::CameraEnvironment>();
    debug_assert_eq!(key.mask, crate::ecs::world::CAMERA_ENVIRONMENT);
    let key = registry.register_serde::<crate::ecs::camera::components::CameraPostProcess>();
    debug_assert_eq!(key.mask, crate::ecs::world::CAMERA_POST_PROCESS);
    let key = registry.register_serde::<crate::ecs::camera::components::Camera>();
    debug_assert_eq!(key.mask, crate::ecs::world::CAMERA);
    let key = registry.register_serde::<crate::ecs::primitives::CastsShadow>();
    debug_assert_eq!(key.mask, crate::ecs::world::CASTS_SHADOW);
    let key = registry.register_serde::<crate::ecs::cloth::components::Cloth>();
    debug_assert_eq!(key.mask, crate::ecs::world::CLOTH);
    let key = registry.register_serde::<crate::ecs::camera::components::ConstrainedAspect>();
    debug_assert_eq!(key.mask, crate::ecs::world::CONSTRAINED_ASPECT);
    let key = registry.register_serde::<crate::ecs::primitives::CullingMask>();
    debug_assert_eq!(key.mask, crate::ecs::world::CULLING_MASK);
    let key = registry.register_serde::<crate::ecs::decal::components::Decal>();
    debug_assert_eq!(key.mask, crate::ecs::world::DECAL);
    let key = registry.register_serde::<crate::ecs::transform::components::GlobalTransform>();
    debug_assert_eq!(key.mask, crate::ecs::world::GLOBAL_TRANSFORM);
    let key = registry.register_serde::<crate::ecs::primitives::Guid>();
    debug_assert_eq!(key.mask, crate::ecs::world::GUID);
    let key = registry.register_serde::<crate::ecs::input::components::Hovered>();
    debug_assert_eq!(key.mask, crate::ecs::world::HOVERED);
    let key = registry.register_serde::<crate::ecs::transform::components::IgnoreParentScale>();
    debug_assert_eq!(key.mask, crate::ecs::world::IGNORE_PARENT_SCALE);
    let key = registry.register_serde::<crate::ecs::mesh::components::InstancedMesh>();
    debug_assert_eq!(key.mask, crate::ecs::world::INSTANCED_MESH);
    let key = registry.register_serde::<crate::ecs::skin::components::Joint>();
    debug_assert_eq!(key.mask, crate::ecs::world::JOINT);
    let key = registry.register_serde::<crate::ecs::light::components::Light>();
    debug_assert_eq!(key.mask, crate::ecs::world::LIGHT);
    let key = registry.register_serde::<crate::ecs::lines::components::Lines>();
    debug_assert_eq!(key.mask, crate::ecs::world::LINES);
    let key = registry.register_serde::<crate::ecs::transform::components::LocalTransform>();
    debug_assert_eq!(key.mask, crate::ecs::world::LOCAL_TRANSFORM);
    let key = registry.register_serde::<crate::ecs::material::components::MaterialRef>();
    debug_assert_eq!(key.mask, crate::ecs::world::MATERIAL_REF);
    let key = registry.register_serde::<crate::ecs::material::components::MaterialVariants>();
    debug_assert_eq!(key.mask, crate::ecs::world::MATERIAL_VARIANTS);
    let key = registry.register_serde::<crate::ecs::morph::components::MorphWeights>();
    debug_assert_eq!(key.mask, crate::ecs::world::MORPH_WEIGHTS);
    let key = registry.register_serde::<crate::ecs::primitives::Name>();
    debug_assert_eq!(key.mask, crate::ecs::world::NAME);
    let key = registry.register_serde::<crate::ecs::camera::components::PanOrbitCamera>();
    debug_assert_eq!(key.mask, crate::ecs::world::PAN_ORBIT_CAMERA);
    let key = registry.register_serde::<nightshade_ecs::dynamic::ChildOf>();
    debug_assert_eq!(key.mask, crate::ecs::world::PARENT);
    let key = registry.register_serde::<crate::render::particles::ParticleEmitter>();
    debug_assert_eq!(key.mask, crate::ecs::world::PARTICLE_EMITTER);
    let key = registry.register_serde::<crate::ecs::prefab::components::PrefabSource>();
    debug_assert_eq!(key.mask, crate::ecs::world::PREFAB_SOURCE);
    let key = registry.register_serde::<crate::render::render_layer::RenderLayer>();
    debug_assert_eq!(key.mask, crate::ecs::world::RENDER_LAYER);
    let key = registry.register_serde::<crate::ecs::mesh::components::RenderMesh>();
    debug_assert_eq!(key.mask, crate::ecs::world::RENDER_MESH);
    let key = registry.register_serde::<crate::ecs::transform::components::Rotation>();
    debug_assert_eq!(key.mask, crate::ecs::world::ROTATION);
    let key = registry.register_serde::<crate::ecs::script::components::Script>();
    debug_assert_eq!(key.mask, crate::ecs::world::SCRIPT);
    let key = registry.register_serde::<crate::ecs::skin::components::Skin>();
    debug_assert_eq!(key.mask, crate::ecs::world::SKIN);
    let key =
        registry.register_serde::<crate::ecs::text::components::TextCharacterBackgroundColors>();
    debug_assert_eq!(
        key.mask,
        crate::ecs::world::TEXT_CHARACTER_BACKGROUND_COLORS
    );
    let key = registry.register_serde::<crate::ecs::text::components::TextCharacterColors>();
    debug_assert_eq!(key.mask, crate::ecs::world::TEXT_CHARACTER_COLORS);
    let key = registry.register_serde::<crate::ecs::text::components::Text>();
    debug_assert_eq!(key.mask, crate::ecs::world::TEXT);
    let key = registry.register_serde::<crate::ecs::camera::components::ThirdPersonCamera>();
    debug_assert_eq!(key.mask, crate::ecs::world::THIRD_PERSON_CAMERA);
    let key = registry.register_serde::<crate::ecs::camera::components::ViewportShading>();
    debug_assert_eq!(key.mask, crate::ecs::world::VIEWPORT_SHADING);
    let key = registry.register_serde::<crate::render::config::ViewportUpdateMode>();
    debug_assert_eq!(key.mask, crate::ecs::world::VIEWPORT_UPDATE_MODE);
    let key = registry.register_serde::<crate::ecs::primitives::Visibility>();
    debug_assert_eq!(key.mask, crate::ecs::world::VISIBILITY);
    let key = registry.register_serde::<crate::ecs::water::components::Water>();
    debug_assert_eq!(key.mask, crate::ecs::world::WATER);
    let key = registry.register_serde::<crate::ecs::vfx::components::Beam>();
    debug_assert_eq!(key.mask, crate::ecs::world::BEAM);
    let key = registry.register_serde::<crate::ecs::vfx::components::LightningBolt>();
    debug_assert_eq!(key.mask, crate::ecs::world::LIGHTNING_BOLT);
    let key = registry.register_serde::<crate::ecs::vfx::components::Trail>();
    debug_assert_eq!(key.mask, crate::ecs::world::TRAIL);
    let key = registry.register_serde::<crate::ecs::vfx::components::VfxAnimator>();
    debug_assert_eq!(key.mask, crate::ecs::world::VFX_ANIMATOR);
    let key = registry.register_serde::<crate::ecs::meshlet::components::MeshletMesh>();
    debug_assert_eq!(key.mask, crate::ecs::world::MESHLET_MESH);
    let key = registry.register_serde::<crate::ecs::camera::components::ProjectionOverride>();
    debug_assert_eq!(key.mask, crate::ecs::world::PROJECTION_OVERRIDE);
    let key = registry.register_serde::<crate::ecs::camera::components::RenderTarget>();
    debug_assert_eq!(key.mask, crate::ecs::world::RENDER_TARGET);
    registry
}

nightshade_ecs::impl_component!(
    crate::ecs::animation::components::AnimationPlayer,
    crate::ecs::primitives::CameraCullingMask,
    crate::ecs::camera::components::CameraEnvironment,
    crate::ecs::camera::components::CameraPostProcess,
    crate::ecs::camera::components::Camera,
    crate::ecs::primitives::CastsShadow,
    crate::ecs::cloth::components::Cloth,
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
    crate::ecs::camera::components::PanOrbitCamera,
    crate::ecs::prefab::components::PrefabSource,
    crate::ecs::mesh::components::RenderMesh,
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
    crate::ecs::camera::components::ProjectionOverride,
    crate::ecs::camera::components::RenderTarget,
);

pub fn build_engine_ecs() -> nightshade_ecs::dynamic::DynEcs {
    let mut ecs = nightshade_ecs::dynamic::DynEcs::new();
    ecs.add_world_at(crate::ecs::world::CORE, register_core_components());
    ecs.add_world_at(
        crate::ecs::world::UI,
        nightshade_ui::register_ui_components(),
    );
    ecs.add_world_at(
        crate::ecs::world::RENDER,
        crate::render::render_world::register_render_components(),
    );
    ecs
}

impl Default for World {
    fn default() -> Self {
        // The capability resources (physics, audio, navmesh, terrain, egui, file
        // watcher) are inserted by their plugins' `build`, so composing the plugin
        // is what enables the capability; engine-core readers skip their work when
        // the resource is absent. PickingWorld (engine-core physics-picking state)
        // and AssetWatcher (asset-file watching) have no owning plugin, so they
        // stay feature-scoped here.
        #[cfg(not(any(
            feature = "physics",
            all(
                feature = "assets",
                feature = "file_watcher",
                not(target_arch = "wasm32")
            ),
        )))]
        let ecs = build_engine_ecs();
        #[cfg(any(
            feature = "physics",
            all(
                feature = "assets",
                feature = "file_watcher",
                not(target_arch = "wasm32")
            ),
        ))]
        let ecs = {
            let mut ecs = build_engine_ecs();
            #[cfg(feature = "physics")]
            ecs.insert_resource(crate::ecs::picking::resources::PickingWorld::default());
            #[cfg(all(
                feature = "assets",
                feature = "file_watcher",
                not(target_arch = "wasm32")
            ))]
            ecs.insert_resource(crate::ecs::asset_watcher::AssetWatcher::default());
            ecs
        };
        let mut ecs = ecs;
        ecs.insert_resource(crate::ecs::text::resources::TextState::default());
        ecs.insert_resource(crate::user_interface::UserInterface::default());
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
        ecs.insert_resource(crate::render::mesh_cache::MeshCache::default());
        ecs.insert_resource(crate::ecs::material::resources::MaterialRegistry::default());
        ecs.insert_resource(crate::ecs::prefab::resources::PrefabCache::default());
        ecs.insert_resource(crate::ecs::prefab::resources::AnimationCache::default());
        ecs.insert_resource(crate::ecs::texture_source::TextureSources::default());
        #[cfg(feature = "scene_graph")]
        ecs.insert_resource(crate::ecs::scene::registry::AssetRegistry::default());
        #[cfg(feature = "scene_graph")]
        ecs.insert_resource(crate::ecs::scene::ScenePrefabs::default());
        #[cfg(feature = "scene_graph")]
        ecs.insert_resource(crate::ecs::scene::hooks::SceneCapabilityHooks::default());
        ecs.insert_resource(crate::ecs::prefab::hooks::PrefabCapabilityHooks::default());
        ecs.insert_resource(crate::ecs::cutscene::resources::CutsceneAudio::default());
        ecs.insert_resource(crate::render::config::RendererState::default());
        ecs.insert_resource(crate::ecs::camera::resources::ActiveCamera::default());
        ecs.insert_resource(crate::ecs::camera::resources::CameraCollision::default());
        ecs.insert_resource(crate::ecs::animation::systems::FootIkGroundQuery::default());
        #[cfg(feature = "grass")]
        ecs.insert_resource(crate::render::grass_config::GrassSettings::default());
        #[cfg(feature = "terrain")]
        ecs.insert_resource(crate::render::terrain_config::TerrainRenderState::default());
        ecs.insert_resource(crate::render::wind::Wind::default());
        ecs.insert_resource(crate::ecs::cloth::resources::ClothMeshSync::default());
        #[cfg(feature = "meshlet")]
        ecs.insert_resource(crate::ecs::meshlet::resources::MeshletAssets::default());
        ecs.insert_resource(crate::ecs::gpu_picking::GpuPicking::default());
        ecs.insert_resource(crate::platform::window::Monitors::default());
        ecs.insert_resource(crate::ecs::tween::Tweens::default());
        ecs.insert_resource(crate::ecs::script::components::GlobalScripts::default());
        ecs.insert_resource(crate::ecs::graphics::selection::Selection::default());
        ecs.insert_resource(crate::render::config::DebugDraw::default());
        ecs.insert_resource(crate::ecs::cutscene::resources::CutsceneDirector::default());
        ecs.insert_resource(crate::ecs::transform::resources::TransformState::default());
        ecs.insert_resource(crate::ecs::graphics::scene_sync::RenderSceneSync::default());
        ecs.insert_resource(crate::ecs::world::scratch::Scratch::default());
        ecs.insert_resource(crate::ecs::scene::typed::ComponentTypeRegistry::default());
        ecs.insert_resource(crate::platform::input::resources::Input::default());
        ecs.insert_resource(crate::platform::input::resources::Gamepad::default());
        ecs.insert_resource(crate::render::config::RenderSettings::default());
        ecs.insert_resource(crate::platform::window::Window::default());
        ecs.insert_resource(crate::viewport::Viewport::default());
        ecs.insert_resource(crate::ecs::time::Time::default());
        ecs.insert_resource(crate::ecs::loading::LoadingState::default());
        ecs.insert_resource(crate::state::PendingState::default());
        nightshade_ui::insert_ui_resources(
            &mut ecs,
            crate::ui::resources::UiWorldIndices { core: CORE, ui: UI },
        );
        Self { ecs }
    }
}
