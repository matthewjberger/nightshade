use crate::components::Watcher;
use crate::resources::Field;
use crate::theme;
use nalgebra_glm::{Vec3, quat_angle_axis, vec3};
use nightshade::prelude::*;
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_scene::contract::LocalTransform;
use nightshade_senses::commands::{hide, make_a_noise};
use nightshade_senses::queries::{alarmed, alertness, last_seen};
use nightshade_senses::{Alert, Exposure, LastSeen, Noises, SensesSettings};

pub fn take_the_orders(
    input: Res<Input>,
    time: Res<Time>,
    settings: Res<SensesSettings>,
    mut field: ResMut<Field>,
    mut noises: ResMut<Noises>,
    mut hiders: Query<(&mut LocalTransform, &mut Exposure)>,
) {
    let Some(hider) = field.hider else {
        return;
    };
    let mut step = vec3(0.0, 0.0, 0.0);
    if input.keyboard.is_key_pressed(KeyCode::ArrowUp) {
        step.z -= 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowDown) {
        step.z += 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowLeft) {
        step.x -= 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowRight) {
        step.x += 1.0;
    }
    field.crouching = input.keyboard.is_key_pressed(KeyCode::ShiftLeft);
    let moving = step.magnitude() > f32::EPSILON;
    let speed = if field.crouching {
        theme::CROUCH_WALK
    } else {
        theme::WALK
    };
    let delta = time.delta_time;
    let crouching = field.crouching;
    let mut here = Vec3::zeros();
    hiders.for_entity(hider, |(transform, exposure)| {
        if moving {
            transform.translation += step.normalize() * speed * delta;
            transform.translation.x = transform.translation.x.clamp(-theme::REACH, theme::REACH);
            transform.translation.z = transform.translation.z.clamp(-theme::REACH, theme::REACH);
        }
        here = transform.translation;
        hide(exposure, &settings, crouching, moving);
    });

    if moving {
        field.footfall += delta;
        if field.footfall >= theme::FOOTFALL {
            field.footfall = 0.0;
            let loudness = if crouching {
                theme::STEP_LOUDNESS * theme::CROUCH_SCALE
            } else {
                theme::STEP_LOUDNESS
            };
            make_a_noise(&mut noises, here, loudness);
        }
    } else {
        field.footfall = 0.0;
    }

    if input.keyboard.just_pressed(KeyCode::Space) {
        field.shouts += 1;
        make_a_noise(&mut noises, here, theme::SHOUT_LOUDNESS);
    }
}

pub fn turn_the_watchers(time: Res<Time>, watchers: Query<(&Watcher, &mut LocalTransform)>) {
    let elapsed = time.uptime_milliseconds as f32 / 1000.0;
    watchers.for_each(|_entity, (watcher, transform)| {
        let offset = watcher.0 as f32 * std::f32::consts::TAU / theme::WATCHERS as f32;
        transform.rotation = quat_angle_axis(offset + elapsed * theme::TURN_RATE, &Vec3::y_axis());
    });
}

pub fn paint_the_watchers(
    settings: Res<SensesSettings>,
    mut materials: ResMut<MaterialRegistry>,
    mut commands: Commands,
    watchers: Query<(&Watcher, &Alert)>,
) {
    let mut wanted = Vec::new();
    watchers.for_each(|entity, (_watcher, alert)| {
        let tint = if alarmed(alert, &settings) {
            theme::WATCHER_ALARMED
        } else {
            theme::WATCHER_CALM
        };
        wanted.push((entity, tint));
    });
    for (entity, tint) in wanted {
        nightshade_graphics::prelude::set_material(
            &mut materials,
            &mut commands,
            entity,
            format!("{}_{}", theme::TITLE, entity.id),
            nightshade_render::data::material::Material {
                base_color: tint,
                roughness: 0.85,
                metallic: 0.0,
                ..Default::default()
            },
        );
    }
}

pub fn show_the_field(
    settings: Res<SensesSettings>,
    field: Res<Field>,
    mut readout: ResMut<Readout>,
    hiders: Query<(&Exposure,)>,
    watchers: Query<(&Alert, &LastSeen)>,
) {
    let mut shown = 0.0_f32;
    hiders.for_each(|_entity, (exposure,)| shown = exposure.0);
    let mut worst = 0.0_f32;
    let mut alarmed_count = 0;
    let mut toward = Vec3::zeros();
    watchers.for_each(|_entity, (alert, seen)| {
        if alertness(alert) > worst {
            worst = alertness(alert);
            toward = last_seen(seen);
        }
        if alarmed(alert, &settings) {
            alarmed_count += 1;
        }
    });
    readout.notes.clear();
    readout.notes.push(format!("exposure {shown:.2}"));
    readout.notes.push(if field.crouching {
        "crouching".to_string()
    } else {
        "standing".to_string()
    });
    readout.notes.push(format!("worst alert {worst:.2}"));
    readout
        .notes
        .push(format!("{alarmed_count} watchers alarmed"));
    readout
        .notes
        .push(format!("they look at {:.1}, {:.1}", toward.x, toward.z));
    readout.notes.push(format!("{} shouts", field.shouts));
}
