use crate::commands::{floor, slab};
use crate::resources::Rack;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_cooldown::{Cooldown, Recharge};
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut rack: ResMut<Rack>,
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

    let mut guns = Vec::new();
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        let start = -(theme::GUNS as f32 - 1.0) * theme::SPACING * 0.5;
        for step in 0..theme::GUNS {
            let gun = slab(
                commands,
                materials,
                vec3(start + step as f32 * theme::SPACING, 0.6, 0.0),
                vec3(1.2, 1.2, 1.2),
                theme::READY_TINT,
            );
            let wait = theme::SHORTEST + step as f32 * theme::STEP;
            commands.queue(move |storage, _resources| {
                storage.set(gun, Cooldown(0.0));
                storage.set(gun, Recharge(wait));
            });
            guns.push(gun);
        }
        spawn_sun(commands);
    };
    rack.guns = guns;
}
