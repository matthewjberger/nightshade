mod plugin;
mod resources;
mod systems;

use nightshade_particles::ParticlePlugin;
use plugin::GalleryPlugin;
use resources::GalleryArguments;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<GalleryArguments>(nightshade_app::Opening::titled(
        "NIGHTSHADE // RETAINED UI GALLERY",
    ));
    app.add_plugin(ParticlePlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(GalleryPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
