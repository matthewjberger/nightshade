use nightshade::prelude::*;

use crate::ecs::GalleryResources;
use crate::systems::{
    animations::run_animation_systems,
    build_ui::{build_gallery_ui, poll_gallery},
};

pub struct GalleryPlugin;

impl Plugin for GalleryPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "NIGHTSHADE // RETAINED UI GALLERY".to_string();
        app.world.res_mut::<Window>().icon_bytes = None;
        app.insert_resource(GalleryResources::default());
        app.add_system(Stage::Startup, initialize);
        app.add_systems(Stage::Update, (poll_gallery, run_animation_systems));
    }
}

fn initialize(gallery_world: &mut GalleryResources, world: &mut World) {
    world.res_mut::<RenderSettings>().render_world_to_swapchain = false;
    world.res_mut::<RenderSettings>().clear_color = [0.0, 0.0, 0.0, 1.0];
    world.res_mut::<RetainedUiState>().background_color = None;

    let camera = spawn_ortho_camera(world, vec2(0.0, 0.0));
    world.res_mut::<ActiveCamera>().0 = Some(camera);

    build_gallery_ui(gallery_world, world);
}
