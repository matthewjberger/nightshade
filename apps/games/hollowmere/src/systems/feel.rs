use crate::commands::effects::spawn_burst;
use crate::resources::{Bursts, Feel, Hero};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::prelude::ThirdPersonCamera;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn bursts(mut bursts: ResMut<Bursts>, mut commands: Commands) {
    for burst in std::mem::take(&mut bursts.pending) {
        spawn_burst(&mut commands, burst);
    }
}

pub fn settle(time: Res<Time>, mut feel: ResMut<Feel>) {
    let delta_time = time.delta_time;
    feel.trauma = (feel.trauma - theme::SHAKE_DECAY * delta_time * feel.trauma.max(0.25)).max(0.0);
    feel.hurt = (feel.hurt - delta_time / theme::HURT_FLASH_SECONDS).max(0.0);
    feel.grey = (feel.grey - delta_time / theme::GREY_SECONDS).max(0.0);
    feel.combat = (feel.combat - delta_time).max(0.0);
    feel.low_pulse += delta_time * 6.0;
    for flash in feel.slot_flash.iter_mut() {
        *flash = (*flash - delta_time / theme::SLOT_FLASH_SECONDS).max(0.0);
    }
}

pub fn shake(
    active: Res<ActiveCamera>,
    feel: Res<Feel>,
    mut hero: ResMut<Hero>,
    mut cameras: Query<(&mut LocalTransform,), With<ThirdPersonCamera>>,
) {
    if feel.trauma <= 0.0 {
        return;
    }
    let Some(camera) = active.0 else {
        return;
    };
    let strength = feel.trauma * feel.trauma * theme::SHAKE_AMOUNT;
    let random = &mut hero.random;
    let offset = Vec3::new(
        (random.f32() - 0.5) * 2.0 * strength,
        (random.f32() - 0.5) * 2.0 * strength,
        (random.f32() - 0.5) * 2.0 * strength * 0.4,
    );
    cameras.for_entity(camera, |(transform,)| {
        transform.translation += offset;
    });
}
