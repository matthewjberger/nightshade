use crate::components::Fade;
use crate::data::look::{FLASH_HEIGHT, FLASH_JITTER, FLASH_SINK};
use crate::data::settings::Settings;
use crate::resources::Flashes;
use nightshade::prelude::*;
use nightshade_config::contract::Config;
use nightshade_platform::prelude::Time;
use nightshade_random::prelude::*;
use nightshade_scene::prelude::LocalTransform;
use nightshade_text::prelude::{Text, TextState};

const SETTLE_RATE: f32 = 9.0;

pub fn show(
    time: Res<Time>,
    config: Res<Config<Settings>>,
    mut flashes: ResMut<Flashes>,
    mut random: ResMut<Random>,
    mut text_state: ResMut<TextState>,
    mut texts: Query<(&mut Text, &mut LocalTransform, &mut Fade)>,
) {
    let flashes = &mut *flashes;
    if flashes.streak_left > 0.0 {
        flashes.streak_left -= time.delta_time;
        if flashes.streak_left <= 0.0 {
            flashes.streak = 0;
        }
    }
    if flashes.pool.is_empty() {
        flashes.pending.clear();
        return;
    }
    let seconds = config.number(Settings::FlashSeconds);
    for flash in std::mem::take(&mut flashes.pending) {
        let entity = flashes.pool[flashes.next];
        flashes.next = (flashes.next + 1) % flashes.pool.len();
        let jitter = Vec3::new(
            between(&mut random, -0.5, 0.5) * FLASH_JITTER,
            between(&mut random, 0.0, 0.4),
            between(&mut random, -0.5, 0.5) * FLASH_JITTER,
        );
        texts.for_entity(entity, |(text, transform, fade)| {
            text_state
                .cache
                .set_text(text.text_index, flash.text.clone());
            text.properties.color = flash.color;
            text.dirty = true;
            transform.translation = flash.position + Vec3::new(0.0, FLASH_HEIGHT, 0.0) + jitter;
            transform.scale = Vec3::new(flash.pop, flash.pop, flash.pop);
            fade.0 = seconds;
        });
        if !flashes.live.contains(&entity) {
            flashes.live.push(entity);
        }
    }
}

pub fn rise(
    time: Res<Time>,
    config: Res<Config<Settings>>,
    mut flashes: ResMut<Flashes>,
    mut texts: Query<(&mut LocalTransform, &mut Fade)>,
) {
    let delta_time = time.delta_time;
    let climb = config.number(Settings::FlashRise);
    let mut live = std::mem::take(&mut flashes.live);
    live.retain(|entity| {
        let mut rising = false;
        texts.for_entity(*entity, |(transform, fade)| {
            if fade.0 <= 0.0 {
                return;
            }
            fade.0 -= delta_time;
            transform.translation.y += climb * delta_time;
            let settle = (delta_time * SETTLE_RATE).min(1.0);
            let scale = transform.scale.x + (1.0 - transform.scale.x) * settle;
            transform.scale = Vec3::new(scale, scale, scale);
            if fade.0 <= 0.0 {
                transform.translation.y = FLASH_SINK;
            } else {
                rising = true;
            }
        });
        rising
    });
    flashes.live = live;
}
