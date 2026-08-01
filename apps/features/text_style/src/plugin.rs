use crate::resources::TextStyleResources;
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct TextStylePlugin;

impl Plugin for TextStylePlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(TextStyleResources::default());
        app.add_system(
            Stage::Startup,
            (systems::setup::initialize, systems::setup::build_panel),
        );
        app.add_system(
            Stage::Update,
            (
                systems::panel::poll,
                systems::style::apply,
                systems::panel::drain_keyboard,
            ),
        );
    }
}
