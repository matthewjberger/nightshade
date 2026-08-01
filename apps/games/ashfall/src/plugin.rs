use crate::data::theme;
use crate::resources::{AshfallResources, Senses};
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct AshfallPlugin;

impl Plugin for AshfallPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(AshfallResources::default());
        app.insert_resource(Senses::default());
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::ask::Ask>::new());
        app.add_system(Stage::Startup, systems::setup::build_overlays);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::map::build);
        app.add_system(Stage::Startup, systems::inspect::build);
        app.add_system(Stage::Startup, systems::effects::upload_the_ash_sprite);
        app.add_system(Stage::Update, systems::menus::update);
        app.add_system(Stage::Update, systems::look::update);
        app.add_system(Stage::Update, systems::hands::sway);
        app.add_system(Stage::Update, systems::player::update);
        app.add_system(Stage::Update, systems::grab::update);
        app.add_system(Stage::Update, systems::grab::land);
        app.add_system(Stage::Update, systems::hazards::update);
        app.add_system(Stage::Update, systems::husks::update);
        app.add_system(Stage::Update, systems::husks::animate);
        app.add_system(Stage::Update, systems::people::attend);
        app.add_system(Stage::Update, systems::pickups::animate);
        app.add_system(Stage::Update, systems::shift::update);
        app.add_system(Stage::FrameUpdate, systems::shift::settle);
        app.add_system(Stage::Update, systems::effects::drift);
        app.add_system(Stage::Update, systems::effects::stain_the_ground);
        app.add_system(Stage::Update, systems::satchel::update);
        app.add_system(Stage::Update, systems::inspect::update);
        app.add_system(Stage::Update, systems::map::update);
        app.add_system(Stage::Update, systems::hud::update);
    }
}
