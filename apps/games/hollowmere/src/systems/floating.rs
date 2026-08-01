use crate::components::Fade;
use crate::resources::{Flashes, Hero};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;
use nightshade_text::prelude::{Text, TextState};

const FLASH_HEIGHT: f32 = 1.9;
const FLASH_JITTER: f32 = 0.6;

pub fn show(
    mut hero: ResMut<Hero>,
    mut flashes: ResMut<Flashes>,
    mut text_state: ResMut<TextState>,
    mut texts: Query<(&mut Text, &mut LocalTransform, &mut Fade)>,
) {
    let flashes = &mut *flashes;
    if flashes.pool.is_empty() {
        flashes.pending.clear();
        return;
    }
    let random = &mut hero.random;
    for flash in std::mem::take(&mut flashes.pending) {
        let entity = flashes.pool[flashes.next];
        flashes.next = (flashes.next + 1) % flashes.pool.len();
        let jitter = Vec3::new(
            (random.f32() - 0.5) * FLASH_JITTER,
            random.f32() * 0.3,
            (random.f32() - 0.5) * FLASH_JITTER,
        );
        texts.for_entity(entity, |(text, transform, fade)| {
            text_state
                .cache
                .set_text(text.text_index, flash.text.clone());
            text.properties.color = flash.color;
            text.dirty = true;
            transform.translation = flash.position + Vec3::new(0.0, FLASH_HEIGHT, 0.0) + jitter;
            transform.scale = Vec3::new(flash.pop, flash.pop, flash.pop);
            fade.0 = theme::FLOATING_SECONDS;
        });
    }
}

pub fn rise(time: Res<Time>, mut texts: Query<(&mut LocalTransform, &mut Fade), With<Text>>) {
    let delta_time = time.delta_time;
    texts.for_each_mut(|_entity, (transform, fade)| {
        if fade.0 <= 0.0 {
            return;
        }
        fade.0 -= delta_time;
        transform.translation.y += theme::FLOATING_RISE * delta_time;
        let settle = (delta_time * 9.0).min(1.0);
        let scale = transform.scale.x + (1.0 - transform.scale.x) * settle;
        transform.scale = Vec3::new(scale, scale, scale);
        if fade.0 <= 0.0 {
            transform.translation.y = -theme::SINK_DEPTH;
        }
    });
}
