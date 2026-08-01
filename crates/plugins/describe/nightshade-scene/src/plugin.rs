use nightshade::app::{App, Stage, push_frame_system};
use nightshade::plugin::Plugin;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        install(app);
        register_frame_systems(&mut app.stages);
    }
}

pub fn install(app: &mut App) {
    app.world
        .resource_or_insert_default::<crate::resources::entity_registry::EntityRegistry>();
    app.world
        .resource_or_insert_default::<crate::resources::transform::TransformState>();
}

pub fn register_frame_systems(stages: &mut nightshade::ecs::Stages<nightshade::ecs::World>) {
    push_frame_system(
        stages,
        Stage::FramePostUpdate,
        crate::systems::update_global_transforms_system,
    );
    push_frame_system(
        stages,
        Stage::FramePostUpdate,
        crate::systems::expire_lifetimes_system,
    );
}
