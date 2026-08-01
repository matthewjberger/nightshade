use crate::components::{Layers, Seed};
use crate::queries::{height_at, shape_of, tint_for};
use crate::resources::Field;
use crate::theme;
use nalgebra_glm::vec2;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, Res, ResMut};
use nightshade_ground::queries::height_at as ground_height;
use nightshade_noise::hash;
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
    mut field: ResMut<Field>,
    storage: &mut Storage,
) {
    let Some(walker) = field.walker else {
        return;
    };
    let (seed, layers) = shape_of(storage).unwrap_or((theme::SEED, theme::LAYERS));
    if input.keyboard.just_pressed(KeyCode::KeyR) {
        storage.set(walker, Seed(hash(seed)));
        field.stale = true;
    }
    if input.keyboard.just_pressed(KeyCode::KeyQ) && layers > 1 {
        storage.set(walker, Layers(layers - 1));
        field.stale = true;
    }
    if input.keyboard.just_pressed(KeyCode::KeyE) && layers < theme::MOST_LAYERS {
        storage.set(walker, Layers(layers + 1));
        field.stale = true;
    }

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
    let step = theme::WALK_RATE * time.delta_time;
    if let Some(transform) = storage.get_mut::<LocalTransform>(walker) {
        transform.translation.x = (transform.translation.x + across * step).clamp(-reach, reach);
        transform.translation.z = (transform.translation.z + along * step).clamp(-reach, reach);
    }
}

pub fn reshape_the_field(
    mut field: ResMut<Field>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    let (stale, columns) = { (field.stale, field.columns.clone()) };
    let (seed, layers) = shape_of(storage).unwrap_or((theme::SEED, theme::LAYERS));
    if !stale {
        return;
    }
    field.stale = false;

    let mut raised = Vec::with_capacity(columns.len());
    for pillar in columns {
        let Some(transform) = storage.get_mut::<LocalTransform>(pillar) else {
            continue;
        };
        let height = height_at(
            transform.translation.x,
            transform.translation.z,
            seed,
            layers,
        );
        transform.translation.y = height * 0.5;
        transform.scale.y = height.max(0.2);
        raised.push((pillar, height));
    }

    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        for (pillar, height) in raised {
            set_material(
                materials,
                commands,
                pillar,
                format!("{}_{}", theme::TITLE, pillar.id),
                Material {
                    base_color: tint_for(height),
                    roughness: 0.9,
                    metallic: 0.0,
                    ..Default::default()
                },
            );
        }
    };
}

pub fn watch_the_walker(
    field: Res<Field>,
    ground: Res<nightshade_ground::Ground>,
    mut readout: ResMut<Readout>,
    storage: &mut Storage,
) {
    let (walker, columns) = { (field.walker, field.columns.len()) };
    let (seed, layers) = shape_of(storage).unwrap_or((theme::SEED, theme::LAYERS));
    let Some(walker) = walker else {
        return;
    };
    let Some(transform) = storage.get::<LocalTransform>(walker) else {
        return;
    };
    let spot = vec2(transform.translation.x, transform.translation.z);
    let standing = ground_height(&ground, storage, spot).unwrap_or(0.0);

    readout.notes.clear();
    readout
        .notes
        .push(format!("{columns} columns, {layers} layers"));
    readout.notes.push(format!("seed {seed}"));
    readout.notes.push(format!(
        "standing at {standing:.2} over {:.1}, {:.1}",
        spot.x, spot.y
    ));
}
