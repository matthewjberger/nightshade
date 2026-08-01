use crate::resources::Sweep;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{EventReader, Res};
use nightshade::prelude::*;
use nightshade_inventory::queries::held;
use nightshade_inventory::{Slots, StackLimit};
use nightshade_pickup::{Collector, PickedUp, Pickup, Taken};
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    time: Res<Time>,
    mut walkers: Query<(&mut LocalTransform, &Collector)>,
) {
    let mut step = Vec3::zeros();
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
    if step == Vec3::zeros() {
        return;
    }
    let walk = step.normalize() * theme::WALK * time.delta_time;
    let edge = theme::REACH - 1.0;
    walkers.for_each_mut(|_entity, (transform, _collector)| {
        transform.translation += walk;
        transform.translation.x = transform.translation.x.clamp(-edge, edge);
        transform.translation.z = transform.translation.z.clamp(-edge, edge);
    });
}

pub fn take_the_coins(taken: EventReader<PickedUp>, limit: Res<StackLimit>, storage: &mut Storage) {
    let mut gone = Vec::new();
    for word in taken.iter() {
        if let Some(slots) = storage.get_mut::<Slots>(word.collector) {
            nightshade_inventory::commands::add(slots, &limit, word.kind, word.count);
        }
        gone.push(word.pickup);
    }
    storage.despawn_entities(&gone);
}

pub fn show_the_purse(sweep: Res<Sweep>, mut readout: ResMut<Readout>, storage: &mut Storage) {
    let Some(collector) = sweep.collector else {
        return;
    };
    let carried = storage
        .get::<Slots>(collector)
        .map(|slots| held(slots, theme::COIN_KIND))
        .unwrap_or(0);
    let reach = storage
        .get::<Collector>(collector)
        .map(|grasp| grasp.0)
        .unwrap_or_default();
    let lying = storage
        .query_ref::<(&Pickup, &Taken)>()
        .iter()
        .filter(|(_entity, (_pickup, gone))| !gone.0)
        .count();
    readout.notes.clear();
    readout.notes.push(format!("{carried} coins"));
    readout.notes.push(format!("{lying} still lying"));
    readout.notes.push(format!("reaching {reach:.1}"));
}
