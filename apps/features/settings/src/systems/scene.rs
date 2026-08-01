use crate::components::Slot;
use crate::data::{BOB_RATE, CUBE_ORIGIN, CUBE_SCALE, PALETTE_COLORS, PALETTE_SHADES, Settings};
use crate::resources::SettingsResources;
use nightshade::prelude::*;
use nightshade_config::contract::Config;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::contract::LocalTransform;

pub fn keep_the_count(
    config: Res<Config<Settings>>,
    mut resources: ResMut<SettingsResources>,
    mut commands: Commands,
) {
    let wanted = config.whole(Settings::Cubes).max(0) as usize;
    if resources.cubes.len() == wanted {
        return;
    }
    while resources.cubes.len() > wanted {
        if let Some(entity) = resources.cubes.pop() {
            commands.despawn_recursive(entity);
        }
    }
    while resources.cubes.len() < wanted {
        let slot = resources.cubes.len();
        let entity = spawn_mesh(&mut commands, "Cube", CUBE_ORIGIN, CUBE_SCALE);
        commands.set(entity, Slot(slot));
        resources.cubes.push(entity);
    }
    resources.shown_palette = None;
    resources.shown_tint = None;
}

pub fn arrange_the_cubes(
    config: Res<Config<Settings>>,
    time: Res<Time>,
    mut resources: ResMut<SettingsResources>,
    cubes: Query<(&mut LocalTransform, &Slot)>,
) {
    let spacing = config.number(Settings::Spacing);
    let bob = config.toggle(Settings::Bob);
    let height = config.number(Settings::BobHeight);
    let middle = (resources.cubes.len().max(1) as f32 - 1.0) * 0.5;
    resources.angle += time.delta_time * config.number(Settings::SpinSpeed);
    let angle = resources.angle;
    cubes.for_each(|_entity, (transform, slot)| {
        let place = slot.0 as f32;
        let rise = if bob {
            (angle * BOB_RATE + place).sin() * height
        } else {
            0.0
        };
        transform.translation = Vec3::new((place - middle) * spacing, 0.5 + rise, 0.0);
        transform.rotation =
            nalgebra_glm::quat_angle_axis(angle + place * 0.3, &Vec3::new(0.0, 1.0, 0.0));
    });
}

pub fn paint_the_cubes(
    config: Res<Config<Settings>>,
    mut resources: ResMut<SettingsResources>,
    mut registry: ResMut<MaterialRegistry>,
    mut commands: Commands,
) {
    let palette = config.choice(Settings::Palette);
    let tint = config.color(Settings::Tint);
    if resources.shown_palette == Some(palette) && resources.shown_tint == Some(tint) {
        return;
    }
    resources.shown_palette = Some(palette);
    resources.shown_tint = Some(tint);
    let color = PALETTE_COLORS[palette.min(PALETTE_COLORS.len() - 1)];
    for (slot, entity) in resources.cubes.iter().copied().enumerate() {
        let shade = PALETTE_SHADES[slot % PALETTE_SHADES.len()];
        set_material(
            &mut registry,
            &mut commands,
            entity,
            format!("settings_cube_{}", entity.id),
            Material {
                base_color: [
                    color[0] * shade * tint[0],
                    color[1] * shade * tint[1],
                    color[2] * shade * tint[2],
                    color[3] * tint[3],
                ],
                ..Default::default()
            },
        );
    }
}
