use crate::data::{
    Ask, CAMERA_DISTANCE, CAMERA_PITCH, CAMERA_TARGET, CAMERA_YAW, CUBE_COLOR, CUBE_ORIGIN,
    CUBE_SCALE, FLOOR_COLOR, FLOOR_ORIGIN, FLOOR_SCALE,
};
use crate::resources::ActionsResources;
use crate::systems::panel;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;
use nightshade_input::Bindings;

pub fn initialize(
    bindings: Res<Bindings<Ask>>,
    mut resources: ResMut<ActionsResources>,
    mut render_settings: ResMut<RenderSettings>,
    mut active_camera: ResMut<ActiveCamera>,
    mut registry: ResMut<MaterialRegistry>,
    mut commands: Commands,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    render_settings.atmosphere = Atmosphere::Sky;
    spawn_sun(&mut commands);

    let camera = spawn_pan_orbit_camera_at(
        &mut commands,
        CAMERA_TARGET,
        CAMERA_DISTANCE,
        CAMERA_YAW,
        CAMERA_PITCH,
        "Actions Camera".to_string(),
    );
    active_camera.0 = Some(camera);

    let floor = spawn_mesh(&mut commands, "Cube", FLOOR_ORIGIN, FLOOR_SCALE);
    set_material(
        &mut registry,
        &mut commands,
        floor,
        "actions_floor".to_string(),
        Material {
            base_color: FLOOR_COLOR,
            ..Default::default()
        },
    );
    let cube = spawn_mesh(&mut commands, "Cube", CUBE_ORIGIN, CUBE_SCALE);
    set_material(
        &mut registry,
        &mut commands,
        cube,
        "actions_cube".to_string(),
        Material {
            base_color: CUBE_COLOR,
            ..Default::default()
        },
    );
    resources.cube = cube;

    panel::build(build.context(), &bindings, &mut resources);
}
