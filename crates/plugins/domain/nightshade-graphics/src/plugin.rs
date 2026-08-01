use nightshade::app::{App, Stage, push_frame_system};
use nightshade::plugin::Plugin;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_platform::contract::input::Input => "PlatformPlugin",
            nightshade_platform::contract::window::Window => "PlatformPlugin",
            nightshade_render::data::config::RenderSettings => "GraphicsPlugin",
            nightshade_render::resources::viewport::Viewport => "GraphicsPlugin",
            nightshade_render::resources::mesh_cache::MeshCache => "GraphicsPlugin",
            nightshade_render::resources::mesh_state::MeshRenderState => "GraphicsPlugin",
            nightshade_render::resources::material_registry::MaterialRegistry => "GraphicsPlugin",
            nightshade_render::resources::texture_cache::TextureCache => "GraphicsPlugin",
            nightshade_render::resources::capture::ScheduledCapture => "GraphicsPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        install(app);
        register_frame_systems(&mut app.stages);
    }
}

pub fn install(app: &mut App) {
    app.add_plugin(nightshade_render::plugin::RenderDescriptionPlugin);
    app.world
        .resource_or_insert_default::<crate::resources::loading::LoadingState>();
    app.world
        .resource_or_insert_default::<crate::resources::queues::CommandQueues>();
    #[cfg(not(target_arch = "wasm32"))]
    app.world
        .resource_or_insert_default::<crate::resources::clusters::ClusterCache>();
}

pub fn register_frame_systems(stages: &mut nightshade::ecs::Stages<nightshade::ecs::World>) {
    push_frame_system(
        stages,
        Stage::FrameStart,
        crate::systems::loading::count_quiet_loading_frames,
    );
    push_frame_system(
        stages,
        Stage::FrameStart,
        crate::systems::viewport::viewport_extent_system,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::systems::bounding_volume::ensure_bounding_volumes,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::systems::light::day_night_cycle_system,
    );
    #[cfg(not(target_arch = "wasm32"))]
    {
        push_frame_system(
            stages,
            Stage::FrameUpdate,
            crate::systems::screenshot::screenshot_key_system,
        );
        push_frame_system(
            stages,
            Stage::FrameUpdate,
            crate::systems::screenshot::scheduled_capture_system,
        );
    }
    push_frame_system(
        stages,
        Stage::FramePostUpdate,
        crate::commands::builtins::process_commands_system,
    );
    #[cfg(not(target_arch = "wasm32"))]
    push_frame_system(
        stages,
        Stage::FramePostUpdate,
        crate::systems::clusters::build_the_clusters,
    );
    push_frame_system(
        stages,
        Stage::FramePostUpdate,
        crate::systems::mesh::update_instanced_mesh_caches_system,
    );
    push_frame_system(
        stages,
        Stage::FramePostUpdate,
        crate::systems::transform_sync::render_sync_transforms_system,
    );
    push_frame_system(
        stages,
        Stage::Render,
        crate::systems::light::render_sync_light_system,
    );
    push_frame_system(
        stages,
        Stage::Render,
        crate::systems::scene_sync::render_sync_scene_system,
    );
    push_frame_system(
        stages,
        Stage::Render,
        crate::systems::material_sync::render_sync_materials_system,
    );
    push_frame_system(
        stages,
        Stage::Render,
        crate::systems::skin::render_sync_skinning_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::commands::builtins::release_dead_texture_owners_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::commands::builtins::cleanup_unused_resources_system,
    );
    stages.stage_mut(Stage::FramePostUpdate.name()).order(
        std::any::type_name_of_val(&crate::commands::builtins::process_commands_system),
        std::any::type_name_of_val(&nightshade_scene::systems::update_global_transforms_system),
    );
    stages.stage_mut(Stage::FramePostUpdate.name()).order(
        std::any::type_name_of_val(&nightshade_scene::systems::update_global_transforms_system),
        std::any::type_name_of_val(&crate::systems::transform_sync::render_sync_transforms_system),
    );
}
