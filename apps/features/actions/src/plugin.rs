use crate::data::{Ask, SOUTHPAW, SOUTHPAW_JUMP, SOUTHPAW_WALK, STORE_FILE, STORE_FOLDER};
use crate::resources::ActionsResources;
use crate::systems;
use nightshade::prelude::*;
use nightshade_input::{Binding, DEFAULT_PROFILE, InputPlugin, Stick};
use nightshade_platform::prelude::Window;

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Actions".to_string();
        app.add_plugin(
            InputPlugin::<Ask>::new()
                .profile(SOUTHPAW)
                .only(Ask::Walk, SOUTHPAW_WALK)
                .bind(Ask::Walk, Binding::Stick(Stick::Left))
                .only(Ask::Jump, SOUTHPAW_JUMP)
                .wearing(DEFAULT_PROFILE)
                .persist(STORE_FOLDER, STORE_FILE),
        );
        app.insert_resource(ActionsResources::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Update, systems::walk::walk_the_cube);
        app.add_system(Stage::Update, systems::profile::swap_the_profile);
        app.add_system(Stage::Update, systems::rebind::rebind_the_jump);
        app.add_system(Stage::Update, systems::panel::refresh_the_panel);
    }
}
