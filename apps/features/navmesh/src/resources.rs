use nightshade::prelude::*;

nightshade::ecs::resource_bundle! {
    pub struct Stage<'a> {
        write {
            active_camera: nightshade_camera::resources::ActiveCamera,
            debug_draw: nightshade_render::data::config::DebugDraw,
            input_capture: nightshade_platform::contract::input_capture::InputCapture,
            navmesh: nightshade_navmesh::resources::NavMeshWorld,
            settings: nightshade_render::data::config::RenderSettings,
            window: nightshade_platform::contract::window::Window,
            loading: nightshade_gltf::Loading,
            queues: nightshade_graphics::resources::queues::CommandQueues,
            materials: nightshade_render::resources::material_registry::MaterialRegistry,
            meshes: nightshade_render::resources::mesh_cache::MeshCache,
            mesh_state: nightshade_render::resources::mesh_state::MeshRenderState,
            textures: nightshade_render::resources::texture_cache::TextureCache,
            sources: nightshade_render::resources::texture_sources::TextureSources,
            load_state: nightshade_graphics::resources::loading::LoadingState,
            hierarchy: nightshade::ecs::HierarchyIndex,
            entities: nightshade_scene::resources::entity_registry::EntityRegistry,
            text_state: nightshade_text::contract::resources::TextState,
            theme: nightshade_ui::resources::ThemeState,
            runtime: nightshade_ui::resources::RetainedUiRuntime,
            groups: nightshade_ui::resources::RetainedUiGroups,
            dirty: nightshade_ui::resources::RetainedUiDirty,
            overlays: nightshade_ui::resources::RetainedUiOverlays,
            interaction: nightshade_ui::resources::RetainedUiInteraction,
            accessibility: nightshade_ui::resources::RetainedUiAccessibility,
            images: nightshade_ui::resources::UiImageLayerAllocator,
        }
        read {
            input: nightshade_platform::contract::input::Input,
            viewport: nightshade_render::resources::viewport::Viewport,
            frame: nightshade_ui::resources::RetainedUiFrame,
            timing: nightshade_ui::resources::RetainedUiTiming,
        }
    }
}

pub struct Scene<'a, 'c> {
    pub storage: &'a mut nightshade::ecs::Storage,
    pub commands: &'a mut Commands<'c>,
    pub stage: Stage<'a>,
}

impl<'a, 'c> Scene<'a, 'c> {
    pub fn prefab_spawn(&mut self) -> nightshade_prefab::PrefabSpawn<'_> {
        nightshade_prefab::PrefabSpawn {
            storage: self.storage,
            parts: nightshade_prefab::PrefabParts {
                meshes: self.stage.meshes,
                mesh_state: self.stage.mesh_state,
                materials: self.stage.materials,
                textures: self.stage.textures,
                sources: self.stage.sources,
                loading: self.stage.load_state,
                hierarchy: self.stage.hierarchy,
            },
        }
    }

    pub fn ui_context(&mut self) -> nightshade_ui::commands::builder::UiBuildContext<'_, 'c> {
        nightshade_ui::commands::builder::UiBuildContext {
            commands: self.commands,
            text_state: self.stage.text_state,
            theme: self.stage.theme,
            runtime: self.stage.runtime,
            groups: self.stage.groups,
            dirty: self.stage.dirty,
            overlays: self.stage.overlays,
            interaction: self.stage.interaction,
            accessibility: self.stage.accessibility,
            images: self.stage.images,
            timing: self.stage.timing,
            frame: self.stage.frame,
            hierarchy: self.stage.hierarchy,
        }
    }
}

pub struct Ground(pub crate::commands::TerrainConfig);

impl Default for Ground {
    fn default() -> Self {
        Self(crate::data::terrain_config())
    }
}

#[derive(Default)]
pub struct Trees {
    pub count: usize,
    pub obstacles: Vec<crate::data::TreeObstacle>,
}

#[derive(Default)]
pub struct Names(pub std::collections::HashSet<String>);

pub struct Behavior {
    pub show_navmesh: bool,
    pub wander: bool,
    pub follow: bool,
    pub followed: Option<Entity>,
    pub last_followed: Option<Entity>,
    pub speed: f32,
}

impl Default for Behavior {
    fn default() -> Self {
        Self {
            show_navmesh: false,
            wander: false,
            follow: false,
            followed: None,
            last_followed: None,
            speed: crate::data::DEFAULT_AGENT_SPEED,
        }
    }
}

#[derive(Default)]
pub struct FoxModel {
    pub asked: Option<nightshade_load::LoadHandle>,
    pub loaded: bool,
    pub clips: crate::data::AnimationIndices,
    pub filtered: Vec<nightshade_animation::contract::AnimationClip>,
    pub skins: Vec<nightshade_prefab::data::ModelSkin>,
    pub prefab: Option<nightshade_prefab::Prefab>,
}

#[derive(Default)]
pub struct Spawns(pub Vec<Vec3>);

#[derive(Default)]
pub struct Viewpoint(pub Option<Entity>);

#[derive(Default, Clone, Copy)]
pub struct Interface {
    pub show_navmesh: Entity,
    pub wander_mode: Entity,
    pub follow_mode: Entity,
    pub algorithm: Entity,
    pub speed: Entity,
    pub spawn_button: Entity,
    pub clear_button: Entity,
    pub foxes_label: Entity,
    pub trees_label: Entity,
    pub triangles_label: Entity,
    pub fps_label: Entity,
    pub following_label: Entity,
}
