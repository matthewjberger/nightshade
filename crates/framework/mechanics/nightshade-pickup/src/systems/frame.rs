use crate::components::{Collector, Middle, Pickup, PickupCount, Taken};
use crate::data::PickedUp;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{EventWriter, Res};
use nightshade_plane::PlayPlane;
use nightshade_plane::queries::across;
use nightshade_scene::contract::LocalTransform;

pub fn gather_the_pickups(
    play_plane: Res<PlayPlane>,
    mut taken: EventWriter<PickedUp>,
    storage: &mut Storage,
) {
    let plane = play_plane.0;
    let collectors: Vec<_> = storage
        .query_ref::<(&LocalTransform, &Collector, Option<&Middle>)>()
        .iter()
        .map(|(entity, (transform, collector, middle))| {
            let lift = middle.map(|middle| middle.0).unwrap_or_default();
            (entity, transform.translation + lift, collector.0)
        })
        .collect();
    if collectors.is_empty() {
        return;
    }
    let lying: Vec<_> = storage
        .query_ref::<(&LocalTransform, &Pickup, &PickupCount, &Taken)>()
        .iter()
        .filter(|(_entity, (_transform, _pickup, _count, gone))| !gone.0)
        .map(|(entity, (transform, pickup, count, _gone))| {
            (entity, transform.translation, pickup.0, count.0)
        })
        .collect();

    let mut gone = Vec::new();
    for (collector, here, reach) in &collectors {
        for (pickup, there, kind, count) in &lying {
            if gone.contains(pickup) {
                continue;
            }
            if across(plane, *here, *there) > *reach {
                continue;
            }
            gone.push(*pickup);
            taken.send(PickedUp {
                collector: *collector,
                pickup: *pickup,
                kind: *kind,
                count: *count,
            });
        }
    }
    for pickup in gone {
        storage.set(pickup, Taken(true));
    }
}
