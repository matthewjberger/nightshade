use crate::commands::pocket::{show_toast, take_item};
use crate::components::{Count, Held, Hop, Spot};
use crate::data;
use crate::queries::cylinder;
use crate::queries::town::lift_at;
use crate::resources::{Pocket, Toast, Town, Village};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn hop_and_gather(
    time: Res<Time>,
    town: Res<Town>,
    village: Res<Village>,
    mut pocket: ResMut<Pocket>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    loose: Query<(&Held, &Count, &Spot, &mut Hop, &mut LocalTransform)>,
) {
    let delta_time = time.delta_time;
    let feet = village.feet;
    let mut taken = Vec::new();
    loose.for_each(|entity, (held, count, spot, hop, transform)| {
        if hop.0 > 0.0 {
            hop.0 = (hop.0 - delta_time * data::PICKUP_HOP_RATE).max(0.0);
            let lift = lift_at(&town, spot.0) + hop.0.sin() * data::PICKUP_HOP;
            transform.translation = cylinder::place(spot.0, lift);
        }
        if cylinder::town_distance(feet, spot.0) < data::PICKUP_REACH {
            taken.push((entity, held.0, count.0));
        }
    });
    if taken.is_empty() {
        return;
    }
    let mut total = 0;
    let mut item = None;
    for (entity, held, count) in taken {
        take_item(&mut pocket, held, count);
        total += count;
        item = Some(held);
        commands.despawn(entity);
    }
    if let Some(item) = item {
        show_toast(&mut toast, format!("Picked up {total} {}", item.name()));
    }
}
