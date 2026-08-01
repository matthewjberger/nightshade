use crate::commands::{floor, slab};
use crate::resources::Rounds;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_health::{Armor, Bounty, Health, MaxHealth, PendingDamage, Shield};
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn line_them_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut rounds: ResMut<Rounds>,
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

    let lined = {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        let mut lined = Vec::new();
        let start = -(theme::BODIES as f32 - 1.0) * theme::SPACING * 0.5;
        for step in 0..theme::BODIES {
            let body = slab(
                commands,
                materials,
                vec3(start + step as f32 * theme::SPACING, 1.0, 0.0),
                vec3(1.0, 2.0, 1.0),
                theme::BODY_TINT,
            );
            let shield = step as f32 * 10.0;
            let armor = step as f32 * 2.0;
            commands.queue(move |storage, _resources| {
                storage.set(body, Health(theme::FULL));
                storage.set(body, MaxHealth(theme::FULL));
                storage.set(body, Shield(shield));
                storage.set(body, Armor(armor));
                storage.set(body, PendingDamage(0.0));
                storage.set(body, Bounty(step as u32));
            });
            lined.push(body);
        }
        spawn_sun(commands);
        lined
    };
    rounds.bodies = lined;
}
