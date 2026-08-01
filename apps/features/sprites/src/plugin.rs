use crate::resources::Farm;
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(Farm::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::player::walk_the_farmer,
                systems::animals::wander,
                systems::spin::turn_the_cube,
                systems::camera::follow_the_farmer,
            ),
        );
    }
}
