use nightshade::prelude::*;

nightshade::ecs::resource_bundle! {
    pub struct Stage<'a> {
        write {
            active_camera: nightshade_camera::resources::ActiveCamera,
            debug_draw: nightshade_render::data::config::DebugDraw,
            settings: nightshade_render::data::config::RenderSettings,
            renderer_state: nightshade_render::data::config::RendererState,
            window: nightshade_platform::contract::window::Window,
            time: nightshade_platform::contract::time::Time,
            physics: nightshade_physics::resources::PhysicsWorld,
            loading: nightshade_gltf::Loading,
            entities: nightshade_scene::resources::entity_registry::EntityRegistry,
            materials: nightshade_render::resources::material_registry::MaterialRegistry,
            meshes: nightshade_render::resources::mesh_cache::MeshCache,
            mesh_state: nightshade_render::resources::mesh_state::MeshRenderState,
            textures: nightshade_render::resources::texture_cache::TextureCache,
            sources: nightshade_render::resources::texture_sources::TextureSources,
            load_state: nightshade_graphics::resources::loading::LoadingState,
            hierarchy: nightshade::ecs::HierarchyIndex,
            text_state: nightshade_text::contract::resources::TextState,
            theme: nightshade_ui::resources::ThemeState,
            runtime: nightshade_ui::resources::RetainedUiRuntime,
            groups: nightshade_ui::resources::RetainedUiGroups,
            dirty: nightshade_ui::resources::RetainedUiDirty,
            overlays: nightshade_ui::resources::RetainedUiOverlays,
            interaction: nightshade_ui::resources::RetainedUiInteraction,
            accessibility: nightshade_ui::resources::RetainedUiAccessibility,
            images: nightshade_ui::resources::UiImageLayerAllocator,
            gamepad_nav: nightshade_ui::resources::RetainedUiGamepadNav,
        }
        read {
            input: nightshade_platform::contract::input::Input,
            gamepad: nightshade_gamepad::contract::Gamepad,
            actions: nightshade_input::contract::Actions<crate::data::Ask>,
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

    pub fn texture_load(&mut self) -> nightshade_graphics::resources::loading::TextureLoad<'_> {
        nightshade_graphics::resources::loading::TextureLoad {
            sources: self.stage.sources,
            cache: self.stage.textures,
            loading: self.stage.load_state,
        }
    }

    pub fn despawn(&mut self, targets: &[nightshade::ecs::Entity]) {
        let Scene { storage, stage, .. } = self;
        let mut caches = nightshade_graphics::commands::builtins::EntityCaches {
            entities: stage.entities,
            meshes: stage.meshes,
            mesh_state: stage.mesh_state,
            textures: stage.textures,
            materials: stage.materials,
            hierarchy: stage.hierarchy,
        };
        nightshade_graphics::commands::builtins::despawn_entities_now(
            storage,
            &mut caches,
            targets,
        );
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
