use crate::theme;
use nightshade::ecs::system_param::{Commands, Res, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_ui::prelude::UiTreeBuilder;
use nightshade_ui_story::{StoryCard, StoryStyle};

pub fn stand_it_up(
    style: Res<StoryStyle>,
    mut card: ResMut<StoryCard>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
    mut builder: nightshade_ui::commands::builder::UiBuild,
) {
    let camera = spawn_pan_orbit_camera_at(
        &mut commands,
        theme::CAMERA_TARGET,
        theme::CAMERA_BACK,
        theme::CAMERA_YAW,
        theme::CAMERA_PITCH,
        "Camera".to_string(),
    );
    active_camera.0 = Some(camera);

    let mut tree = UiTreeBuilder::new(builder.context());
    nightshade_ui_story::commands::build(&mut tree, &style, &mut card);
    tree.finish();
}
