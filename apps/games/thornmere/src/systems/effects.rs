use crate::commands::effects::spawn_burst;
use crate::data::{
    GREY_SECONDS, HURT_FLASH_SECONDS, SHAKE_AMOUNT, SHAKE_DECAY, SLOT_FLASH_SECONDS,
};
use crate::queries::night_amount;
use crate::resources::{Bursts, Feel, ThornmereResources};
use crate::tags::Lamp;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::prelude::ThirdPersonCamera;
use nightshade_platform::prelude::*;
use nightshade_render::data::config::RendererState;
use nightshade_scene::prelude::{LocalTransform, Visibility};

pub fn bursts(mut bursts: ResMut<Bursts>, mut commands: Commands) {
    for burst in std::mem::take(&mut bursts.pending) {
        spawn_burst(&mut commands, burst);
    }
}

pub fn settle(time: Res<Time>, mut feel: ResMut<Feel>) {
    let delta_time = time.delta_time;
    feel.trauma = (feel.trauma - SHAKE_DECAY * delta_time * feel.trauma.max(0.25)).max(0.0);
    feel.hurt = (feel.hurt - delta_time / HURT_FLASH_SECONDS).max(0.0);
    feel.grey = (feel.grey - delta_time / GREY_SECONDS).max(0.0);
    feel.combat = (feel.combat - delta_time).max(0.0);
    feel.low_pulse += delta_time * 6.0;
    for flash in feel.slot_flash.iter_mut() {
        *flash = (*flash - delta_time / SLOT_FLASH_SECONDS).max(0.0);
    }
}

pub fn shake(
    active: Res<ActiveCamera>,
    feel: Res<Feel>,
    mut resources: ResMut<ThornmereResources>,
    mut cameras: Query<(&mut LocalTransform,), With<ThirdPersonCamera>>,
) {
    if feel.trauma <= 0.0 {
        return;
    }
    let Some(camera) = active.0 else {
        return;
    };
    let strength = feel.trauma * feel.trauma * SHAKE_AMOUNT;
    let random = &mut resources.random;
    let offset = Vec3::new(
        (random.f32() - 0.5) * 2.0 * strength,
        (random.f32() - 0.5) * 2.0 * strength,
        (random.f32() - 0.5) * 2.0 * strength * 0.4,
    );
    cameras.for_entity(camera, |(transform,)| {
        transform.translation += offset;
    });
}

pub fn lamps(
    renderer_state: Res<RendererState>,
    mut feel: ResMut<Feel>,
    lamps: Query<(&Visibility,), WithTag<Lamp>>,
    mut visibilities: Query<(&mut Visibility,)>,
) {
    let lit = night_amount(renderer_state.day_night.hour) > 0.35;
    if feel.lamps_lit == Some(lit) {
        return;
    }
    feel.lamps_lit = Some(lit);
    let mut lamp_entities = Vec::new();
    lamps.for_each(|entity, _| lamp_entities.push(entity));
    for entity in lamp_entities {
        visibilities.for_entity(entity, |(visibility,)| visibility.visible = lit);
    }
}
