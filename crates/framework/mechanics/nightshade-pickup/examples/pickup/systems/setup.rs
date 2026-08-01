use crate::commands::{floor, slab};
use crate::resources::Sweep;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_inventory::{Selected, Slots, StackLimit};
use nightshade_pickup::{Collector, Pickup, PickupCount};
use nightshade_random::Random;
use nightshade_random::commands::disc;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn scatter_the_coins(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut stack_limit: ResMut<StackLimit>,
    mut sweep: ResMut<Sweep>,
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
    let mut roll = Random::seeded(11);
    let hand;
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        for _step in 0..theme::COINS {
            let spot = disc(&mut roll, theme::RING);
            let coin = slab(
                commands,
                materials,
                vec3(spot.x, 0.25, spot.y),
                vec3(0.5, 0.5, 0.5),
                theme::COIN_TINT,
            );
            commands.queue(move |storage, _resources| {
                storage.set(coin, Pickup(theme::COIN_KIND));
                storage.set(coin, PickupCount(1));
                storage.set(coin, nightshade_pickup::Taken(false));
            });
        }
        let collector = slab(
            commands,
            materials,
            vec3(theme::RING, 0.6, 0.0),
            vec3(1.2, 1.2, 1.2),
            theme::COLLECTOR_TINT,
        );
        commands.queue(move |storage, _resources| {
            storage.set(collector, Collector(theme::GRASP));
            let mut slots = Slots::default();
            nightshade_inventory::commands::make_room(&mut slots, 4);
            storage.set(collector, slots);
            storage.set(collector, Selected(0));
        });
        hand = Some(collector);
        spawn_sun(commands);
    };
    stack_limit.0 = 64;
    sweep.collector = hand;
}
