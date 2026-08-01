use crate::resources::{Handles, Pictures, Progress, Screen};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(Handles::default());
        app.insert_resource(Progress::default());
        app.insert_resource(Pictures::default());
        app.add_plugin(StatePlugin::new(Screen::Loading));
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.on_enter(Screen::Loading, systems::screens::show_the_card);
        app.on_enter(Screen::Playing, systems::screens::show_the_scene);
        app.add_system(
            Stage::Update,
            while_in(Screen::Loading, systems::watch::watch_the_loads),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::reload::reload_on_click),
        );
    }
}
