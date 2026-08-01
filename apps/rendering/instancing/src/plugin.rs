use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct InstancingPlugin;

impl Plugin for InstancingPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Update, (systems::title::show_the_rate,));
    }
}
