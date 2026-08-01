use crate::resources::Bounce;
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct TracedBouncePlugin;

impl Plugin for TracedBouncePlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(Bounce::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::toggle::take_the_ask,
                systems::toggle::settle_the_grid,
                systems::title::show_the_rate,
            ),
        );
    }
}
