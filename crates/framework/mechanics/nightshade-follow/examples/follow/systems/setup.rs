use crate::commands::{floor, slab};
use crate::resources::Line;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_follow::{FollowOffset, Following};
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut line: ResMut<Line>,
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

    let lead;
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        let head = slab(
            commands,
            materials,
            vec3(theme::RING, 0.8, 0.0),
            vec3(1.2, 1.4, 1.2),
            theme::LEAD_TINT,
        );
        let mut ahead = head;
        for step in 0..theme::TAIL {
            let tail = slab(
                commands,
                materials,
                vec3(theme::RING, 0.8, step as f32 * theme::GAP),
                vec3(0.9, 1.0, 0.9),
                theme::TAIL_TINT,
            );
            commands.queue(move |storage, _resources| {
                storage.set(tail, Following(ahead));
                storage.set(tail, FollowOffset(nalgebra_glm::vec3(0.0, 0.0, theme::GAP)));
            });
            ahead = tail;
        }
        lead = Some(head);
        spawn_sun(commands);
    };
    line.lead = lead;
}
