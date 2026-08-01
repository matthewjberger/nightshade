use crate::commands::effects::sparkle;
use crate::commands::pocket::{add_item, show_toast};
use crate::components::{Layer, Upright};
use crate::data::Item;
use crate::queries::sheet::standing;
use crate::resources::{Diorama, Inventory, Sheet, Toast};
use crate::tags::Egg;
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn lay_the_eggs(
    time: Res<Time>,
    sheet: Res<Sheet>,
    mut commands: Commands,
    chickens: Query<(&mut Layer, &LocalTransform)>,
) {
    let delta_time = time.delta_time;
    chickens.for_each(|_entity, (layer, transform)| {
        layer.until_next -= delta_time;
        if layer.until_next > 0.0 {
            return;
        }
        layer.until_next = theme::EGG_INTERVAL;
        let (region, height) = standing(&sheet, theme::EGG);
        let sprite = Sprite::new(theme::ATLAS, region)
            .with_size(0.5, 0.5 * height)
            .with_anchor(Vec2::new(0.5, 0.0))
            .with_layer(theme::LAYER_STANDING);
        let position = transform.translation + Vec3::new(0.3, 0.0, 0.2);
        let egg = spawn_sprite(&mut commands, sprite, position);
        commands.add_tag_type::<Egg>(egg);
        commands.set(egg, Upright::default());
        commands.set(
            egg,
            crate::components::Lit {
                base: [1.0, 1.0, 1.0, 1.0],
            },
        );
    });
}

pub fn gather_the_eggs(
    diorama: Res<Diorama>,
    mut inventory: ResMut<Inventory>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    eggs: Query<(&LocalTransform,), WithTag<Egg>>,
) {
    let feet = diorama.feet;
    let mut picked = Vec::new();
    eggs.for_each(|entity, (transform,)| {
        let gap = transform.translation - feet;
        if Vec2::new(gap.x, gap.z).magnitude() < theme::EGG_PICKUP_RADIUS {
            picked.push((entity, transform.translation));
        }
    });
    for (entity, position) in picked {
        commands.despawn(entity);
        add_item(&mut inventory, Item::Egg, 1);
        sparkle(&mut commands, position + Vec3::new(0.0, 0.3, 0.0));
        show_toast(&mut toast, "Found an egg");
    }
}
