use crate::commands::collect;
use crate::components::{Phase, PickupKind, RestingHeight};
use crate::contract::BoomstickEvent;
use crate::data;
use crate::resources::BoomstickResources;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn update(
    time: Res<Time>,
    mut resources: ResMut<BoomstickResources>,
    mut events: EventWriter<BoomstickEvent>,
    mut commands: Commands,
    loot: Query<(&PickupKind, &RestingHeight, &Phase, &mut LocalTransform)>,
) {
    let clock = time.uptime_seconds as f32;
    let Some(at) = resources.player.at else {
        return;
    };

    let mut touched: Vec<(Entity, PickupKind)> = Vec::new();
    loot.for_each(|entity, (kind, resting, phase, transform)| {
        transform.rotation =
            nalgebra_glm::quat_angle_axis(clock * data::PICKUP_SPIN + phase.0, &Vec3::y());
        transform.translation.y = resting.0 + (clock * 2.0 + phase.0).sin() * data::PICKUP_BOB;
        if (transform.translation - at).magnitude() < data::PICKUP_RADIUS {
            touched.push((entity, *kind));
        }
    });

    for (entity, kind) in touched {
        if !collect(&mut resources.player, kind) {
            continue;
        }
        commands.despawn_recursive(entity);
        events.send(BoomstickEvent::PickedUp {
            label: label_of(kind).to_string(),
        });
    }
}

fn label_of(kind: PickupKind) -> &'static str {
    match kind {
        PickupKind::Medkit => "medkit",
        PickupKind::Stimpack => "stimpack",
        PickupKind::Plate => "armor plate",
        PickupKind::Shells => "shells",
        PickupKind::Bullets => "bullets",
        PickupKind::Rockets => "rockets",
        PickupKind::Cells => "cells",
        PickupKind::RedKey => "the red key",
        PickupKind::BlueKey => "the blue key",
        PickupKind::Chaingun => "the chaingun",
        PickupKind::Launcher => "the rocket launcher",
        PickupKind::Plasma => "the plasma gun",
    }
}
