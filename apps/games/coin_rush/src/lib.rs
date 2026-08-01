mod data;
mod plugin;
mod resources;
mod systems;
mod theme;

use plugin::CoinRushPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled(
        "Coin Rush",
    ))
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(nightshade_gamepad::GamepadPlugin)
    .add_plugin(CoinRushPlugin)
    .run()
}

nightshade_platform::android_main!(launch);
