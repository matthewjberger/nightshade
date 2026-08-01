use crate::commands::flashes::score_hit;
use crate::components::{Cell, Fallen, Glow, Pivot, Prop, Toppling};
use crate::data::look::FLASH_PROP_BASE;
use crate::data::motion::FALLEN_TILT;
use crate::data::settings::Settings;
use crate::resources::{City, Flashes, Knocks};
use nightshade::prelude::*;
use nightshade_config::contract::Config;
use nightshade_ease::prelude::smooth_step;
use nightshade_platform::prelude::Time;
use nightshade_render::components::light::Light;
use nightshade_scene::prelude::LocalTransform;

pub fn knock(
    config: Res<Config<Settings>>,
    mut knocks: ResMut<Knocks>,
    mut city: ResMut<City>,
    mut flashes: ResMut<Flashes>,
    mut commands: Commands,
    props: Query<(&Cell, &Prop), Without<Light>>,
    lights: Query<(&Cell, &Prop, &Light)>,
) {
    if knocks.pending.is_empty() {
        return;
    }
    let combo = config.number(Settings::ComboSeconds);
    let mut felled = Vec::new();
    for knock in knocks.pending.drain(..) {
        let Some(layout) = city.cells.get_mut(&knock.key) else {
            continue;
        };
        let Some(slot) = layout.props.get_mut(knock.index) else {
            continue;
        };
        let Some(lot) = slot.take() else {
            continue;
        };
        let center = (lot.minimum + lot.maximum) * 0.5;
        let half = (lot.maximum - lot.minimum) * 0.5;
        let reach = half.x * knock.direction.x.abs() + half.y * knock.direction.z.abs();
        let pivot = Vec3::new(
            center.x + knock.direction.x * reach,
            0.0,
            center.y + knock.direction.z * reach,
        );
        score_hit(
            &mut flashes,
            Vec3::new(center.x, 0.0, center.y),
            FLASH_PROP_BASE,
            combo,
        );
        felled.push((knock.key, knock.index, knock.direction, pivot));
    }
    props.for_each(|entity, (cell, prop)| {
        for (key, index, direction, pivot) in &felled {
            if cell.0 == *key && prop.0 == *index {
                commands.set(entity, Toppling(*direction));
                commands.set(entity, Pivot(*pivot));
                commands.set(entity, Fallen(0.0));
            }
        }
    });
    lights.for_each(|entity, (cell, prop, light)| {
        for (key, index, _, _) in &felled {
            if cell.0 == *key && prop.0 == *index {
                commands.set(
                    entity,
                    Light {
                        intensity: 0.0,
                        ..light.clone()
                    },
                );
                commands.set(entity, Glow(0.0));
            }
        }
    });
}

pub fn topple(
    time: Res<Time>,
    config: Res<Config<Settings>>,
    mut commands: Commands,
    mut falling: Query<(&Toppling, &mut Pivot, &mut Fallen, &mut LocalTransform)>,
) {
    let seconds = config.number(Settings::ToppleSeconds).max(0.05);
    let slide = config.number(Settings::KnockSlide);
    let delta_time = time.delta_time;
    let mut down = Vec::new();
    falling.for_each_mut(|entity, (toppling, pivot, fallen, transform)| {
        if fallen.0 >= 1.0 {
            down.push(entity);
            return;
        }
        let before = fallen.0;
        let progress = delta_time / seconds;
        fallen.0 = (before + progress).min(1.0);
        let angle = (smooth_step(fallen.0) - smooth_step(before)) * FALLEN_TILT;
        let axis = Vec3::y().cross(&toppling.0);
        if axis.norm() < 1e-4 {
            return;
        }
        let turn = nalgebra_glm::quat_angle_axis(angle, &axis.normalize());
        let step = toppling.0 * (slide * 2.0 * (1.0 - before) * progress);
        pivot.0 += step;
        let arm = transform.translation + step - pivot.0;
        transform.translation = pivot.0 + nalgebra_glm::quat_rotate_vec3(&turn, &arm);
        transform.rotation = turn * transform.rotation;
    });
    for entity in down {
        commands.queue(move |storage, _| {
            storage.remove_components::<(Toppling, Pivot, Fallen)>(entity);
        });
    }
}
