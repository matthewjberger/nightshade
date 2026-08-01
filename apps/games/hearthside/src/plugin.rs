use crate::contract::HearthsideCommands;
use crate::data::Ask;
use crate::resources::{Grid, Lot, Models};
use crate::systems;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;

pub struct HearthsidePlugin;

impl Plugin for HearthsidePlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Hearthside".to_string();
        app.insert_resource(Lot::default());
        app.insert_resource(Grid::default());
        app.insert_resource(Models::default());
        app.insert_resource(HearthsideCommands::default());
        app.insert_resource(systems::shell::new_shell());
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_plugin(nightshade_shell::ShellOverlayPlugin::<
            crate::resources::HearthsideShell,
        >::default());
        app.add_system(Stage::Startup, systems::setup::dress_lot);
        app.add_system(Stage::Startup, systems::setup::request_models);
        app.add_system(Stage::Startup, systems::setup::build_lot);
        app.add_system(Stage::Startup, systems::interface::build);
        app.add_system(
            Stage::Update,
            (
                systems::shell::route_keys,
                systems::shell::route_the_asks,
                systems::models::collect,
                systems::models::raise_dweller,
                systems::build::aim,
                systems::build::choose,
                systems::build::place,
                systems::dweller::decide,
                systems::dweller::walk,
                systems::dweller::use_piece,
                systems::dweller::turn,
                systems::dweller::animate,
                systems::needs::decay,
                systems::needs::earn,
                systems::needs::apply_commands,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::interface::show_money,
                systems::interface::show_needs,
                systems::interface::show_palette,
                systems::interface::show_status,
            ),
        );
    }
}
