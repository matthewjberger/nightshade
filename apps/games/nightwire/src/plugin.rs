use crate::contract::NightwireCommands;
use crate::data::Ask;
use crate::glass;
use crate::resources::NightwireResources;
use crate::systems;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_wgpu::RenderAppExt;

pub struct NightwirePlugin;

impl Plugin for NightwirePlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Nightwire".to_string();
        app.insert_resource(NightwireResources {
            glass_on: true,
            glass_amount: 1.0,
            ..Default::default()
        });
        app.insert_resource(NightwireCommands::default());
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::setup::build_screens);
        app.add_system(Stage::Startup, systems::setup::apply_arguments);
        app.add_system(
            Stage::Update,
            (
                systems::menus::update,
                systems::switchboard::apply,
                systems::transcript::reveal,
                systems::screens::apply,
                systems::glass::update,
                systems::board::refresh,
            ),
        );
        app.on_pre_render(glass::install);
        app.on_pre_render(glass::drive);
    }
}
