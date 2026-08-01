use crate::resources::Probes;
use crate::theme;
use nalgebra_glm::{Vec3, vec2};
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, Res, ResMut};
use nightshade_ground::queries::{height_at, solid_at};
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
    probes: Res<Probes>,
    storage: &mut Storage,
) {
    let Some(walker) = probes.walker else {
        return;
    };
    let mut across = 0.0;
    let mut along = 0.0;
    if input.keyboard.is_key_pressed(KeyCode::ArrowLeft) {
        across -= 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowRight) {
        across += 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowUp) {
        along -= 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowDown) {
        along += 1.0;
    }
    if across == 0.0 && along == 0.0 {
        return;
    }
    let reach = theme::COLUMNS as f32 * theme::STEP * 0.5;
    let step = theme::RATE * time.delta_time;
    if let Some(transform) = storage.get_mut::<LocalTransform>(walker) {
        transform.translation.x = (transform.translation.x + across * step).clamp(-reach, reach);
        transform.translation.z = (transform.translation.z + along * step).clamp(-reach, reach);
    }
}

pub fn probe_the_ground(
    ground: Res<nightshade_ground::Ground>,
    mut probes_held: ResMut<Probes>,
    mut materials: ResMut<MaterialRegistry>,
    mut readout: ResMut<Readout>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    let probes = probes_held.probes.clone();
    let walker = probes_held.walker;

    let mut asked = Vec::with_capacity(probes.len());
    for probe in &probes {
        let Some(transform) = storage.get::<LocalTransform>(*probe) else {
            continue;
        };
        asked.push((
            *probe,
            vec2(transform.translation.x, transform.translation.z),
        ));
    }

    let mut solid = 0;
    let mut standing = 0.0;
    let mut under = vec2(0.0, 0.0);
    let mut inside = false;
    for (probe, spot) in asked {
        let high = height_at(&ground, storage, spot).unwrap_or(0.0);
        let point = Vec3::new(spot.x, high + theme::HOVER, spot.y);
        let here = solid_at(&ground, storage, point);
        if here {
            solid += 1;
        }
        if Some(probe) == walker {
            standing = high;
            under = spot;
            inside = here;
        }
        if let Some(transform) = storage.get_mut::<LocalTransform>(probe) {
            transform.translation = point;
        }
    }

    if probes_held.inside != inside {
        probes_held.inside = inside;
        if let Some(walker) = walker {
            let tint = if inside {
                theme::BURIED_TINT
            } else {
                theme::PROBE_TINT
            };
            set_material(
                &mut materials,
                &mut commands,
                walker,
                format!("{}_{}", theme::TITLE, walker.id),
                Material {
                    base_color: tint,
                    roughness: 0.85,
                    metallic: 0.0,
                    ..Default::default()
                },
            );
        }
    }

    readout.notes.clear();
    readout.notes.push(format!(
        "riding at {standing:.2} over {:.1}, {:.1}",
        under.x, under.y
    ));
    readout.notes.push(
        if inside {
            "standing in something solid"
        } else {
            "standing in the open"
        }
        .to_string(),
    );
    readout
        .notes
        .push(format!("{solid} of {} probes are buried", probes.len()));
}
