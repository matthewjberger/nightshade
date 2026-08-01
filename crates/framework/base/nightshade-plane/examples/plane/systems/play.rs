use crate::resources::Walk;
use crate::theme;
use nalgebra_glm::{Vec3, vec2};
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, Res, ResMut};
use nightshade_plane::commands::move_across;
use nightshade_plane::queries::{across, flatten, lift, raise};
use nightshade_plane::{Plane, PlayPlane};
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_render::commands::material_registry::set_material;
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    time: Res<Time>,
    mut play: ResMut<PlayPlane>,
    walk: Res<Walk>,
    storage: &mut Storage,
) {
    if input.keyboard.just_pressed(KeyCode::Space) {
        play.0 = match play.0 {
            Plane::Ground => Plane::Side,
            Plane::Side => Plane::Ground,
        };
    }

    let Some(walker) = walk.walker else {
        return;
    };
    let mut wanted = vec2(0.0, 0.0);
    if input.keyboard.is_key_pressed(KeyCode::ArrowLeft) {
        wanted.x -= 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowRight) {
        wanted.x += 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowUp) {
        wanted.y += 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowDown) {
        wanted.y -= 1.0;
    }
    if wanted.x == 0.0 && wanted.y == 0.0 {
        return;
    }
    let step = raise(play.0, wanted * theme::RATE * time.delta_time);
    if let Some(transform) = storage.get_mut::<LocalTransform>(walker) {
        let mut spot = transform.translation;
        move_across(play.0, &mut spot, step);
        if across(play.0, Vec3::zeros(), spot) <= theme::RING {
            transform.translation = spot;
        }
    }
}

pub fn watch_the_walker(
    play_plane: Res<PlayPlane>,
    mut walk: ResMut<Walk>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut readout: ResMut<Readout>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    let plane = play_plane.0;
    let Some(walker) = walk.walker else {
        return;
    };
    let Some(transform) = storage.get::<LocalTransform>(walker) else {
        return;
    };
    let spot = transform.translation;
    let flat = flatten(plane, spot);
    let reach = across(plane, Vec3::zeros(), spot);
    let held = lift(plane, spot);

    if walk.shown != plane {
        walk.shown = plane;
        let tint = match plane {
            Plane::Ground => theme::GROUND_TINT,
            Plane::Side => theme::SIDE_TINT,
        };
        {
            let commands = &mut commands;

            let materials = &mut *material_registry;
            set_material(
                materials,
                commands,
                walker,
                format!("{}_{}", theme::TITLE, walker.id),
                Material {
                    base_color: tint,
                    roughness: 0.85,
                    metallic: 0.0,
                    ..Default::default()
                },
            );
        };
    }

    let name = match plane {
        Plane::Ground => "ground",
        Plane::Side => "side",
    };
    readout.notes.clear();
    readout.notes.push(format!("{name} plane"));
    readout
        .notes
        .push(format!("across {:.1}, {:.1}", flat.x, flat.y));
    readout
        .notes
        .push(format!("{reach:.1} from the middle, {held:.1} off it"));
}
