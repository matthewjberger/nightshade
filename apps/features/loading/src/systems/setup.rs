use crate::commands::interface;
use crate::commands::loads::ask_for_everything;
use crate::resources::{Handles, Pictures, Progress};
use crate::theme;
use nightshade::prelude::*;
use nightshade_audio::resources::Sounds;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;
use nightshade_ui::prelude::UiTreeBuilder;

pub fn initialize(
    mut settings: ResMut<RenderSettings>,
    mut active_camera: ResMut<ActiveCamera>,
    mut handles: ResMut<Handles>,
    mut progress: ResMut<Progress>,
    mut models: ResMut<nightshade_gltf::Loading>,
    mut pictures: ResMut<Pictures>,
    mut sounds: ResMut<Sounds>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::Nebula;
    handles.sun = spawn_sun(&mut commands);
    handles.camera = spawn_pan_orbit_camera_at(
        &mut commands,
        theme::CAMERA_TARGET,
        theme::CAMERA_DISTANCE,
        theme::CAMERA_YAW,
        theme::CAMERA_PITCH,
        "Camera".to_string(),
    );
    active_camera.0 = Some(handles.camera);

    ask_for_everything(
        &mut progress,
        &mut handles,
        &mut models,
        &mut pictures,
        &mut sounds,
    );

    let mut tree = UiTreeBuilder::new(build.context());
    interface::build(&mut tree, &mut handles);
    tree.finish();
}
