use crate::commands::{floor, slab};
use crate::resources::Bag;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_inventory::commands::make_room;
use nightshade_inventory::{Selected, Slots};
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut bag: ResMut<Bag>,
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

    let holder;
    let mut posts = Vec::new();
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        let start = -(theme::SLOTS as f32 - 1.0) * theme::SPACING * 0.5;
        for step in 0..theme::SLOTS {
            posts.push(slab(
                commands,
                materials,
                vec3(start + step as f32 * theme::SPACING, 0.2, 0.0),
                vec3(1.4, 0.4, 1.4),
                theme::SLOT_TINT,
            ));
        }
        let body = slab(
            commands,
            materials,
            vec3(0.0, 1.0, -4.0),
            vec3(1.2, 2.0, 1.2),
            theme::HELD_TINT,
        );
        commands.queue(move |storage, _resources| {
            let mut slots = Slots::default();
            make_room(&mut slots, theme::SLOTS);
            storage.set(body, slots);
            storage.set(body, Selected(0));
        });
        holder = Some(body);
        spawn_sun(commands);
    };
    bag.holder = holder;
    bag.posts = posts;
}
