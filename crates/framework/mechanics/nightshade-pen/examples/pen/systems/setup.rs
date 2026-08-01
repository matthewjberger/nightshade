use crate::commands::{floor, slab};
use crate::resources::Yard;
use crate::theme;
use nalgebra_glm::vec2;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_pen::Pen;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut yard: ResMut<Yard>,
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
    let free;
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        for (across, along) in [
            (theme::ARENA, 0.0),
            (-theme::ARENA, 0.0),
            (0.0, theme::ARENA),
            (0.0, -theme::ARENA),
        ] {
            let wide = if across == 0.0 {
                theme::ARENA * 2.0
            } else {
                0.3
            };
            let deep = if along == 0.0 {
                theme::ARENA * 2.0
            } else {
                0.3
            };
            slab(
                commands,
                materials,
                vec3(across, 0.4, along),
                vec3(wide, 0.8, deep),
                theme::WALL_TINT,
            );
        }
        let held = slab(
            commands,
            materials,
            vec3(0.0, 0.8, 0.0),
            vec3(1.2, 1.2, 1.2),
            theme::WALKER_TINT,
        );
        commands.queue(move |storage, _resources| {
            storage.set(held, Pen(vec2(theme::ARENA, theme::ARENA)));
        });
        walker = Some(held);
        free = Some(slab(
            commands,
            materials,
            vec3(0.0, 2.4, 0.0),
            vec3(0.8, 0.8, 0.8),
            theme::FREE_TINT,
        ));
        spawn_sun(commands);
    };
    yard.walker = walker;
    yard.free = free;
}
