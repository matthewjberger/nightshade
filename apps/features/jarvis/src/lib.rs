mod components;
mod plugin;
mod queries;
mod resources;
mod systems;
mod theme;

use plugin::JarvisPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled(
        "Jarvis",
    ))
    .add_plugin(nightshade_openxr::OpenXrPlugin::default())
    .add_plugin(nightshade_particles::ParticlePlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(JarvisPlugin)
    .run()
}
