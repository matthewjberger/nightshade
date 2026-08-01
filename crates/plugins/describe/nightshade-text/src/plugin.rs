use nightshade::World;
use nightshade::app::{App, Stage, push_frame_system};
use nightshade::plugin::Plugin;

pub struct TextPlugin;

impl Plugin for TextPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_render::resources::mesh_state::MeshRenderState => "GraphicsPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::contract::resources::TextState::default());
        app.insert_resource(crate::resources::TextSyncState::default());
        app.add_system(
            Stage::Render,
            crate::systems::sync::mark_text_sync_dirty_system,
        );
        app.add_system(Stage::Render, crate::systems::sync::render_sync_text_system);
    }
}

pub fn register_frame_systems(stages: &mut nightshade::ecs::Stages<World>) {
    push_frame_system(
        stages,
        Stage::Render,
        crate::systems::sync::mark_text_sync_dirty_system,
    );
    push_frame_system(
        stages,
        Stage::Render,
        crate::systems::sync::render_sync_text_system,
    );
}
