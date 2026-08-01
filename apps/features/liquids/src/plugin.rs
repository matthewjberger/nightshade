use crate::resources::{Digging, Seed, Watcher};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::Window;
use nightshade_voxels::VoxelsPlugin;

pub struct LiquidsPlugin;

impl Plugin for LiquidsPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(Seed::default());
        app.insert_resource(Watcher::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::viewer::aim_the_viewer,
                systems::meet::set_the_stone,
            ),
        );
        app.add_plugin(VoxelsPlugin::<Digging>::default());
    }
}
