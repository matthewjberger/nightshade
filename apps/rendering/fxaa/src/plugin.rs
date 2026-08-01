use crate::resources::{FxaaArguments, Opening, Panel};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct FxaaPlugin;

impl Plugin for FxaaPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(Panel::default());
        let arguments = app.world.res::<FxaaArguments>();
        let smoothing_on = !arguments.off;
        let toggle = arguments.toggle;
        let driver = toggle.then(|| {
            let mut driver = nightshade_ui_test::UiTestDriver::new("fxaa");
            driver.set_exit_on_complete(false);
            driver
                .at_frame(theme::TOGGLE_FRAME)
                .click_entity(theme::SWITCH_TEST_ID);
            driver
                .wait(theme::TOGGLE_SETTLE)
                .assert_value_bool(theme::SWITCH_TEST_ID, !smoothing_on);
            driver.start();
            driver
        });
        app.insert_resource(Opening {
            smoothing_on,
            driver,
        });
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (systems::panel::rehearse, systems::panel::drive),
        );
    }
}
