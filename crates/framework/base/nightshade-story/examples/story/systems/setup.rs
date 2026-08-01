use crate::commands::{floor, slab};
use crate::resources::Board;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::system_param::{Commands, Res, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_story::prelude::Story;

pub fn stand_it_up(
    story: Res<Story>,
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut board: ResMut<Board>,
    mut commands: Commands,
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

    let materials = &mut *material_registry;
    floor(&mut commands, materials, theme::REACH);
    spawn_sun(&mut commands);

    let rooms = story.rooms.len() as f32;
    for index in 0..story.rooms.len() {
        let across = (index as f32 - (rooms - 1.0) * 0.5) * theme::ROOM_SPAN;
        board.rooms.push(slab(
            &mut commands,
            materials,
            vec3(across, theme::ROOM_LIFT, -2.0),
            vec3(2.2, 0.6, 2.2),
            theme::ROOM_TINT,
        ));
    }

    let things = story.things.len() as f32;
    for index in 0..story.things.len() {
        let across = (index as f32 - (things - 1.0) * 0.5) * theme::THING_SPAN;
        board.things.push(slab(
            &mut commands,
            materials,
            vec3(across, theme::THING_LIFT, 3.0),
            vec3(1.2, 1.2, 1.2),
            theme::GONE_TINT,
        ));
    }
}
