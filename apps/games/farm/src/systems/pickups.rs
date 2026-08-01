use crate::commands::pocket::{add_item, show_toast};
use crate::components::Count;
use crate::data::Item;
use crate::resources::{Diorama, Inventory, Toast};
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_scene::prelude::*;

pub fn gather_the_pickups(
    diorama: Res<Diorama>,
    mut inventory: ResMut<Inventory>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    loose: Query<(&Item, &Count, &LocalTransform)>,
) {
    let feet = Vec2::new(diorama.feet.x, diorama.feet.z);
    let mut taken = Vec::new();
    loose.for_each(|entity, (item, count, transform)| {
        let spot = Vec2::new(transform.translation.x, transform.translation.z);
        if (spot - feet).magnitude() < theme::PICKUP_REACH {
            taken.push((entity, *item, count.0));
        }
    });
    if taken.is_empty() {
        return;
    }
    let mut total = 0;
    let mut item = None;
    for (entity, held, count) in taken {
        add_item(&mut inventory, held, count);
        total += count;
        item = Some(held);
        commands.despawn(entity);
    }
    if let Some(item) = item {
        show_toast(&mut toast, format!("Picked up {} {}", total, item.name()));
    }
}
