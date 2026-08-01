use crate::data::Settings;
use crate::resources::SdfScene;
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_config::prelude::ConfigPlugin;
use nightshade_platform::prelude::*;

pub struct SdfDemoPlugin;

impl Plugin for SdfDemoPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.add_plugin(ConfigPlugin::<Settings>::new());
        app.insert_resource(SdfScene::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::views::start);
        app.add_system(Stage::First, (systems::tools::select, systems::tools::aim));
        app.add_system(
            Stage::Update,
            (
                systems::tunnel::aim,
                systems::views::cycle,
                systems::views::rescale,
                systems::props::release,
                systems::stones::throw,
            ),
        );
        app.add_system(Stage::FrameUpdate, systems::shapes::pose);
        app.add_system(
            Stage::Update,
            (
                systems::tools::carve,
                systems::tools::build,
                systems::tools::draw,
                systems::tools::drag,
                systems::ground::carry,
                systems::hud::show_rate,
                systems::hud::show_view,
                systems::hud::show_tool,
            ),
        );
    }
}
