use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::*;

use crate::resources::{GalleryArguments, GalleryResources};
use crate::systems::{
    animations::run_animation_systems,
    build_ui::{build_gallery_ui, poll_gallery, requested_screen},
};

pub struct GalleryPlugin;

impl Plugin for GalleryPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "NIGHTSHADE // RETAINED UI GALLERY".to_string();
        app.world.res_mut::<Window>().icon_bytes = None;
        app.insert_resource(GalleryResources::default());
        app.add_system(Stage::Startup, initialize);
        app.add_system(Stage::Startup, build_gallery_ui);
        app.add_system(Stage::Startup, start_gallery);
        app.add_system(Stage::Update, (poll_gallery, run_animation_systems));
    }
}

fn initialize(
    mut settings: ResMut<RenderSettings>,
    mut runtime: ResMut<RetainedUiRuntime>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
) {
    settings.render_world_to_swapchain = false;
    settings.clear_color = [0.0, 0.0, 0.0, 1.0];
    runtime.background_color = None;

    active_camera.0 = Some(spawn_ortho_camera(&mut commands, vec2(0.0, 0.0)));
}

fn start_gallery(mut gallery: ResMut<GalleryResources>, arguments: Res<GalleryArguments>) {
    gallery.state.pending_screen = requested_screen(arguments.screen.as_deref());
}
