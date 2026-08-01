use crate::commands::{floor, slab};
use crate::resources::Stack;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut stack: ResMut<Stack>,
    mut commands: Commands,
) {
    let camera = {
        let commands = &mut commands;

        spawn_pan_orbit_camera_at(
            commands,
            theme::CAMERA_TARGET,
            theme::CAMERA_BACK,
            theme::CAMERA_YAW,
            theme::CAMERA_PITCH,
            "Camera".to_string(),
        )
    };
    active_camera.0 = Some(camera);

    let mut posts = Vec::with_capacity(theme::POSTS);
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        let span = (theme::POSTS as f32 - 1.0) * theme::ROW_GAP * 0.5;
        for step in 0..theme::POSTS {
            posts.push(slab(
                commands,
                materials,
                vec3(
                    step as f32 * theme::ROW_GAP - span,
                    theme::POST_LOW * 0.5,
                    0.0,
                ),
                vec3(theme::POST_WIDE, theme::POST_LOW, theme::POST_WIDE),
                theme::RUNNER_TINT,
            ));
        }
        spawn_sun(commands);
    };
    stack.posts = posts;
}
