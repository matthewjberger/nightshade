use crate::resources::{Aim, Cursor, Digging, Plan, UiHandles, Watcher};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::Window;
use nightshade_voxels::VoxelsPlugin;

pub struct DemolitionPlugin;

impl Plugin for DemolitionPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(Plan);
        app.insert_resource(Watcher::default());
        app.insert_resource(Aim::default());
        app.insert_resource(Cursor::default());
        app.insert_resource(UiHandles::default());
        app.add_system(
            Stage::Startup,
            (systems::setup::initialize, systems::interface::build),
        );
        app.add_system(
            Stage::Update,
            (
                systems::controls::fly,
                systems::blast::aim,
                systems::blast::blast,
                systems::blast::rebuild,
                systems::controls::keep_the_cursor,
                systems::interface::refresh,
            ),
        );
        app.add_plugin(VoxelsPlugin::<Digging>::default());
    }
}
