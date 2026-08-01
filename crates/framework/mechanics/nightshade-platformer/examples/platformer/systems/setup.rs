use crate::commands::{floor, slab};
use crate::resources::Jumper;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_platformer::{Airborne, Coyote, JumpAsked};
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut jumper: ResMut<Jumper>,
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

    let walker;
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        slab(
            commands,
            materials,
            vec3(theme::LEDGE, theme::LEDGE_TOP * 0.5, 0.0),
            vec3(6.0, theme::LEDGE_TOP, 4.0),
            theme::LEDGE_TINT,
        );
        let body = slab(
            commands,
            materials,
            vec3(-6.0, 1.0, 0.0),
            vec3(1.0, 2.0, 1.0),
            theme::WALKER_TINT,
        );
        commands.queue(move |storage, _resources| {
            storage.set(body, Airborne(false));
            storage.set(body, Coyote(0.0));
            storage.set(body, JumpAsked(0.0));
        });
        walker = Some(body);
        spawn_sun(commands);
    };
    jumper.walker = walker;
}
