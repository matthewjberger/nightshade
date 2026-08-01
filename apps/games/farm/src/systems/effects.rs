use crate::commands::effects::{burst, launch_shell};
use crate::commands::life::{spawn_rocks, spawn_tree};
use crate::components::{Bend, Bush, Flip, Phase, Remaining, Shake, Stump};
use crate::data::Errand;
use crate::data::TreeKind;
use crate::tags::{Firefly, Rocks};

use crate::queries::pocket::is_done;
use crate::queries::sheet::standing;
use crate::queries::tools::swing_angle;
use crate::queries::weather::plant_lean;
use crate::resources::{Diorama, Fireworks, Quarry, Quests, Sheet, Weather};
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_platform::prelude::*;
use nightshade_render::data::config::RendererState;
use nightshade_render::prelude::night_factor;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::Sprite;

pub fn tend_the_fireflies(
    renderer_state: Res<RendererState>,
    weather: Res<Weather>,
    fireflies: Query<(&mut ParticleEmitter,), WithTag<Firefly>>,
) {
    let out =
        night_factor(renderer_state.day_night.hour, &theme::TWILIGHT) > 0.5 && !weather.raining;
    fireflies.for_each(|_entity, (emitter,)| {
        emitter.enabled = out;
    });
}

pub fn shake_the_trees(
    time: Res<Time>,
    mut commands: Commands,
    shaking: Query<(&mut Shake, &mut Sprite)>,
) {
    let delta_time = time.delta_time;
    let mut settled = Vec::new();
    shaking.for_each(|entity, (shake, sprite)| {
        shake.remaining -= delta_time;
        if shake.remaining <= 0.0 {
            sprite.lean = 0.0;
            settled.push(entity);
            return;
        }
        sprite.lean = (shake.remaining * theme::SHAKE_RATE).sin()
            * theme::SHAKE_LEAN
            * (shake.remaining / theme::SHAKE_SECONDS);
    });
    for entity in settled {
        commands.remove::<(Shake,)>(entity);
    }
}

pub fn sway_the_plants(
    weather: Res<Weather>,
    plants: Query<(&Phase, &Bend, Option<&Shake>, &mut Sprite)>,
) {
    let wind = weather.wind;
    let gust = weather.gust;
    plants.for_each(|_entity, (phase, bend, shake, sprite)| {
        if shake.is_some() {
            return;
        }
        sprite.lean = plant_lean(phase.0, bend.0, wind, gust);
    });
}

pub fn regrow_the_stumps(
    diorama: Res<Diorama>,
    sheet: Res<Sheet>,
    mut commands: Commands,
    stumps: Query<(&Stump, &LocalTransform)>,
) {
    let day = diorama.day;
    let mut grown = Vec::new();
    stumps.for_each(|entity, (stump, transform)| {
        if day.saturating_sub(stump.since_day) >= theme::REGROW_DAYS {
            grown.push((entity, transform.translation));
        }
    });
    for (entity, position) in grown {
        commands.despawn_recursive(entity);
        spawn_tree(&mut commands, &sheet, position, TreeKind::Small);
    }
}

pub fn restock_the_quarry(
    diorama: Res<Diorama>,
    sheet: Res<Sheet>,
    mut quarry: ResMut<Quarry>,
    mut commands: Commands,
    rocks: Query<(&LocalTransform,), WithTag<Rocks>>,
) {
    if quarry.restocked_day == Some(diorama.day) {
        return;
    }
    quarry.restocked_day = Some(diorama.day);
    let mut standing_rocks = Vec::new();
    rocks.for_each(|_entity, (transform,)| {
        standing_rocks.push(Vec2::new(transform.translation.x, transform.translation.z));
    });
    for spot in quarry.spots.clone() {
        let taken = standing_rocks
            .iter()
            .any(|rock| (*rock - spot).magnitude() < 0.3);
        if !taken {
            spawn_rocks(
                &mut commands,
                &sheet,
                nalgebra_glm::Vec3::new(spot.x, theme::QUARRY_HEIGHT as f32, spot.y),
            );
        }
    }
}

pub fn ripen_the_bushes(
    diorama: Res<Diorama>,
    sheet: Res<Sheet>,
    bushes: Query<(&mut Bush, &mut Sprite)>,
) {
    let day = diorama.day;
    bushes.for_each(|_entity, (bush, sprite)| {
        if bush.picked_day.is_some_and(|picked| picked != day) {
            bush.picked_day = None;
            let (region, height) = standing(&sheet, theme::BERRY_BUSH);
            sprite.region = region;
            sprite.size.y = height;
        }
    });
}

pub fn swing_the_tools(
    time: Res<Time>,
    mut commands: Commands,
    swings: Query<(&mut Remaining, &Flip, &mut LocalTransform)>,
) {
    let delta_time = time.delta_time;
    let mut done = Vec::new();
    swings.for_each(|entity, (remaining, flip, transform)| {
        remaining.0 -= delta_time;
        if remaining.0 <= 0.0 {
            done.push(entity);
            return;
        }
        let progress = 1.0 - remaining.0 / theme::SWING_SECONDS;
        transform.rotation =
            nalgebra_glm::quat_angle_axis(swing_angle(progress, flip.0), &nalgebra_glm::Vec3::z());
    });
    for entity in done {
        commands.despawn(entity);
    }
}

pub fn light_the_sky(
    time: Res<Time>,
    renderer_state: Res<RendererState>,
    quests: Res<Quests>,
    weather: Res<Weather>,
    mut fireworks: ResMut<Fireworks>,
    mut commands: Commands,
    mut trails: Query<(&mut ParticleEmitter,)>,
) {
    let hour = renderer_state.day_night.hour;
    let show_time = is_done(&quests, Errand::Carrots)
        && !weather.raining
        && (hour >= theme::FIREWORKS_HOUR || hour < theme::FIREWORKS_END)
        && night_factor(hour, &theme::TWILIGHT) > 0.9;
    let delta_time = time.delta_time;

    if show_time {
        fireworks.until_next -= delta_time;
        if fireworks.until_next <= 0.0 {
            fireworks.until_next = theme::FIREWORKS_INTERVAL;
            launch_shell(&mut commands, &mut fireworks);
        }
    }

    let mut bursts = Vec::new();
    for shell in &mut fireworks.shells {
        shell.fuse -= delta_time;
        shell.velocity.y -= 9.81 * delta_time;
        shell.position += shell.velocity * delta_time;
        let position = shell.position;
        let done = shell.fuse <= 0.0;
        trails.for_entity(shell.entity, |(emitter,)| {
            emitter.position = position;
            if done {
                emitter.enabled = false;
            }
        });
        if done {
            bursts.push((position, shell.color, shell.willow));
        }
    }
    fireworks.shells.retain(|shell| shell.fuse > 0.0);

    for (position, color, willow) in bursts {
        burst(&mut commands, position, color, willow);
    }
}
