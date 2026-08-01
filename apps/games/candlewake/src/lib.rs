mod data;
mod plugin;
mod systems;
mod theme;

use plugin::CandlewakePlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled(theme::TITLE).sized(theme::WINDOW),
    )
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(CandlewakePlugin)
    .run()
}

nightshade_platform::android_main!(launch);
