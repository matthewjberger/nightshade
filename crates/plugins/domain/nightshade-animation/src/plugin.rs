use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::resources::ChannelCache::default());
        app.insert_resource(crate::resources::FrozenJoints::default());
        app.add_system(
            Stage::FrameUpdate,
            crate::systems::evaluate_animation_graphs,
        );
        app.add_system(Stage::FrameUpdate, crate::systems::evaluate_blend_trees);
        app.add_system(Stage::FrameUpdate, crate::systems::update_animation_players);
        app.add_system(Stage::FrameUpdate, crate::systems::apply_animations);
        app.add_system(Stage::FrameUpdate, crate::systems::apply_root_motion);
        app.add_system(
            Stage::Render,
            crate::systems::sync::render_sync_animation_system,
        );
    }
}
