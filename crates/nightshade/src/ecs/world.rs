pub mod components {
    pub use crate::ecs::animation::components::{
        AnimationChannel, AnimationClip, AnimationInterpolation, AnimationPlayer,
        AnimationProperty, AnimationSampler, AnimationSamplerOutput, AnimationValue,
    };
    pub use crate::ecs::audio::components::{AudioListener, AudioSource};
    pub use crate::ecs::bounding_volume::components::{BoundingVolume, OrientedBoundingBox};
    pub use crate::ecs::camera::components::{
        Camera, OrthographicCamera, PanOrbitCamera, PerspectiveCamera, Projection, Smoothing,
    };
    pub use crate::ecs::input::components::Hovered;
    pub use crate::ecs::light::components::{Light, LightType};
    pub use crate::ecs::lines::components::{Line, Lines, MAX_LINES};
    pub use crate::ecs::material::components::{AlphaMode, Material, MaterialRef};
    pub use crate::ecs::mesh::components::{InstanceTransform, InstancedMesh, RenderMesh};
    pub use crate::ecs::morph::components::MorphWeights;
    pub use crate::ecs::name::components::Name;
    pub use crate::ecs::particles::components::{
        ColorGradient, EmitterShape, EmitterType, ParticleEmitter, ParticleSystemStats,
    };
    pub use crate::ecs::physics::components::{
        CharacterControllerComponent, ColliderComponent, ColliderShape, PhysicsInterpolation,
        RigidBodyComponent,
    };
    pub use crate::ecs::render_layer::components::RenderLayer;
    pub use crate::ecs::script::components::{Script, ScriptSource};
    pub use crate::ecs::shadow::components::CastsShadow;
    pub use crate::ecs::skin::components::{Joint, Skin};
    pub use crate::ecs::sprite::components::Sprite;
    pub use crate::ecs::text::components::{
        HudAnchor, HudText, Text, TextAlignment, TextProperties, VerticalAlignment,
    };
    pub use crate::ecs::transform::components::{
        GlobalTransform, LocalTransform, LocalTransformDirty, Parent, Rotation,
    };
    pub use crate::ecs::visibility::components::Visibility;
}

pub mod resources {
    #[cfg(feature = "audio")]
    pub use crate::ecs::audio::resources::AudioEngine;
    pub use crate::ecs::event_bus::resources::{EventBus, Message};
    pub use crate::ecs::graphics::resources::{
        Atmosphere, ColorGrading, ColorGradingPreset, Fog, Graphics, TonemapAlgorithm, VertexSnap,
    };
    pub use crate::ecs::input::resources::{Input, MouseState};
    pub use crate::ecs::material::resources::MaterialRegistry;
    pub use crate::ecs::physics::resources::PhysicsWorld;
    pub use crate::ecs::prefab::resources::MeshCache;
    #[cfg(feature = "scripting")]
    pub use crate::ecs::script::resources::ScriptRuntime;
    pub use crate::ecs::text::resources::TextCache;
    pub use crate::ecs::ui::resources::UserInterface;
    pub use crate::ecs::window::resources::{ViewportRect, Window, WindowTiming};
    pub use crate::render::wgpu::texture_cache::TextureCache;
}

pub mod events {
    pub use crate::ecs::event_bus::resources::{InputEvent, KeyState, Message};
}

pub mod commands;
pub use commands::{
    CommandQueue, WorldCommand, cleanup_unused_resources_system, load_hdr_skybox,
    load_procedural_textures, process_commands_system, set_material_with_textures,
    spawn_3d_billboard_text_with_properties, spawn_3d_text_with_properties, spawn_cone_at,
    spawn_cube_at, spawn_cylinder_at, spawn_instanced_mesh, spawn_instanced_mesh_with_material,
    spawn_mesh_at, spawn_plane_at, spawn_sphere_at, spawn_sun, spawn_torus_at,
};

pub mod animation_systems {
    pub use crate::ecs::animation::systems::{apply_animations, update_animation_players};
}

mod render_trait;
pub use render_trait::Render;

pub use freecs::Entity;
pub use nalgebra_glm::{Mat4, Quat, Vec2, Vec3, Vec4};

freecs::ecs! {
    World {
        animation_player: crate::ecs::animation::components::AnimationPlayer => ANIMATION_PLAYER,
        name: crate::ecs::name::components::Name => NAME,
        local_transform: crate::ecs::transform::components::LocalTransform => LOCAL_TRANSFORM,
        global_transform: crate::ecs::transform::components::GlobalTransform => GLOBAL_TRANSFORM,
        local_transform_dirty: crate::ecs::transform::components::LocalTransformDirty => LOCAL_TRANSFORM_DIRTY,
        parent: crate::ecs::transform::components::Parent => PARENT,
        ignore_parent_scale: crate::ecs::transform::components::IgnoreParentScale => IGNORE_PARENT_SCALE,
        audio_source: crate::ecs::audio::components::AudioSource => AUDIO_SOURCE,
        audio_listener: crate::ecs::audio::components::AudioListener => AUDIO_LISTENER,
        camera: crate::ecs::camera::components::Camera => CAMERA,
        pan_orbit_camera: crate::ecs::camera::components::PanOrbitCamera => PAN_ORBIT_CAMERA,
        light: crate::ecs::light::components::Light => LIGHT,
        lines: crate::ecs::lines::components::Lines => LINES,
        visibility: crate::ecs::visibility::components::Visibility => VISIBILITY,
        render_mesh: crate::ecs::mesh::components::RenderMesh => RENDER_MESH,
        material_ref: crate::ecs::material::components::MaterialRef => MATERIAL_REF,
        render_layer: crate::ecs::render_layer::components::RenderLayer => RENDER_LAYER,
        sprite: crate::ecs::sprite::components::Sprite => SPRITE,
        text: crate::ecs::text::components::Text => TEXT,
        hud_text: crate::ecs::text::components::HudText => HUD_TEXT,
        bounding_volume: crate::ecs::bounding_volume::components::BoundingVolume => BOUNDING_VOLUME,
        hovered: crate::ecs::input::components::Hovered => HOVERED,
        rotation: crate::ecs::transform::components::Rotation => ROTATION,
        casts_shadow: crate::ecs::shadow::components::CastsShadow => CASTS_SHADOW,
        rigid_body: crate::ecs::physics::components::RigidBodyComponent => RIGID_BODY,
        collider: crate::ecs::physics::components::ColliderComponent => COLLIDER,
        physics_material: crate::ecs::physics::material::PhysicsMaterialComponent => PHYSICS_MATERIAL,
        character_controller: crate::ecs::physics::components::CharacterControllerComponent => CHARACTER_CONTROLLER,
        physics_interpolation: crate::ecs::physics::components::PhysicsInterpolation => PHYSICS_INTERPOLATION,
        instanced_mesh: crate::ecs::mesh::components::InstancedMesh => INSTANCED_MESH,
        particle_emitter: crate::ecs::particles::components::ParticleEmitter => PARTICLE_EMITTER,
        prefab_source: crate::ecs::prefab::components::PrefabSource => PREFAB_SOURCE,
        script: crate::ecs::script::components::Script => SCRIPT,
        skin: crate::ecs::skin::components::Skin => SKIN,
        joint: crate::ecs::skin::components::Joint => JOINT,
        morph_weights: crate::ecs::morph::components::MorphWeights => MORPH_WEIGHTS,
        navmesh_agent: crate::ecs::navmesh::components::NavMeshAgent => NAVMESH_AGENT,
    }
    Resources {
        window: crate::ecs::window::resources::Window,
        user_interface: crate::ecs::ui::resources::UserInterface,
        immediate_ui: crate::ecs::ui::immediate::ImmediateUi,
        graphics: crate::ecs::graphics::resources::Graphics,
        input: crate::ecs::input::resources::Input,
        #[cfg(feature = "audio")]
        audio: crate::ecs::audio::resources::AudioEngine,
        physics: crate::ecs::physics::resources::PhysicsWorld,
        navmesh: crate::ecs::navmesh::resources::NavMeshWorld,
        text_cache: crate::ecs::text::resources::TextCache,
        mesh_cache: crate::ecs::prefab::resources::MeshCache,
        material_registry: crate::ecs::material::resources::MaterialRegistry,
        texture_cache: crate::render::wgpu::texture_cache::TextureCache,
        pending_font_loads: Vec<crate::ecs::text::commands::PendingFontLoad>,
        active_camera: Option<freecs::Entity>,
        event_bus: crate::ecs::event_bus::resources::EventBus,
        command_queue: Vec<crate::ecs::world::commands::WorldCommand>,
        children_cache: std::collections::HashMap<freecs::Entity, Vec<freecs::Entity>>,
        children_cache_valid: bool,
        cleanup_frame_counter: u64,
        dropped_files: Vec<crate::ecs::input::resources::DroppedFile>,
        skinning_offsets: std::collections::HashMap<freecs::Entity, usize>,
        total_skinning_joints: u32,
        #[cfg(feature = "scripting")]
        script_runtime: crate::ecs::script::resources::ScriptRuntime,
        #[cfg(feature = "openxr")]
        xr: crate::xr::XrResources,
        #[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
        steam: crate::steam::SteamResources,
        picking_world: crate::ecs::picking::resources::PickingWorld,
    }
}

pub use crate::ecs::input::resources::DroppedFile;
