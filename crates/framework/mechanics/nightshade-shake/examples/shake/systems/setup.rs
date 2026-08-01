use crate::commands::{floor, slab};
use crate::components::{Post, Rest};
use crate::resources::Hammer;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_shake::{ShakeOffset, ShakeTurn, Trauma};

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut hammer_held: ResMut<Hammer>,
    mut commands: Commands,
    _storage: &mut Storage,
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

    let hammer;
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        let start = -(theme::POSTS as f32 - 1.0) * theme::SPACING * 0.5;
        for step in 0..theme::POSTS {
            let rest = vec3(start + step as f32 * theme::SPACING, 1.2, 0.0);
            let post = slab(
                commands,
                materials,
                rest,
                vec3(1.0, 2.4, 1.0),
                theme::POST_TINT,
            );
            commands.queue(move |storage, _resources| {
                storage.set(post, Trauma(0.0));
                storage.set(post, ShakeOffset(nalgebra_glm::Vec3::zeros()));
                storage.set(post, ShakeTurn(0.0));
                storage.set(post, Rest(rest));
                storage.set(post, Post(step));
            });
        }
        hammer = Some(slab(
            commands,
            materials,
            vec3(start, theme::HAMMER_HEIGHT, 0.0),
            vec3(1.4, 0.5, 1.4),
            theme::HAMMER_TINT,
        ));
        spawn_sun(commands);
    };
    hammer_held.entity = hammer;
}
