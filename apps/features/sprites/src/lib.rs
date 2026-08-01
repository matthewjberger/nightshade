mod components;
mod plugin;
mod resources;
mod systems;
mod theme;

use plugin::SpritesPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled(theme::TITLE).settling_at(theme::SETTLED),
    );
    app.add_plugin(nightshade_sprite::SpritePlugin)
        .add_plugin(SpritesPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
