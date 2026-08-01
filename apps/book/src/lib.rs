mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;

use plugin::WebsitePlugin;
use resources::BookArguments;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<BookArguments>(
        nightshade_app::Opening::titled("The Nightshade Book")
            .settling_at(data::book::CAPTURE_FRAME),
    )
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(WebsitePlugin)
    .run()
}

nightshade_platform::android_main!(launch);
