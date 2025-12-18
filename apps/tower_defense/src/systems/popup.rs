use crate::ecs::{GameWorld, MONEY_POPUP, MoneyPopup};
use nightshade::prelude::*;

pub fn spawn_money_popup(
    game_world: &mut GameWorld,
    world: &mut World,
    position: Vec3,
    amount: i32,
) {
    let (text, color) = if amount > 0 {
        (
            format!("+${}", amount),
            nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0),
        )
    } else {
        (
            format!("-${}", -amount),
            nalgebra_glm::vec4(1.0, 0.0, 0.0, 1.0),
        )
    };

    let text_index = world.resources.text_cache.add_text(&text);
    let text_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | TEXT | VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.get_name_mut(text_entity) {
        *name = Name("Money Popup".to_string());
    }

    if let Some(transform) = world.get_local_transform_mut(text_entity) {
        transform.translation = position + nalgebra_glm::vec3(0.0, 1.5, 0.0);
    }

    if let Some(text_component) = world.get_text_mut(text_entity) {
        text_component.text_index = text_index;
        text_component.properties = TextProperties {
            font_size: 36.0,
            color,
            alignment: TextAlignment::Center,
            outline_width: 0.08,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };
        text_component.dirty = true;
    }

    let game_entity = game_world.spawn_entities(MONEY_POPUP, 1)[0];
    game_world.set_money_popup(
        game_entity,
        MoneyPopup {
            text_entity,
            lifetime: 0.0,
        },
    );
}

pub fn update_money_popups(game_world: &mut GameWorld, world: &mut World, delta_time: f32) {
    let entities: Vec<_> = game_world.query_entities(MONEY_POPUP).collect();
    let mut popups_to_remove = Vec::new();

    for entity in entities {
        if let Some(mut popup) = game_world.get_money_popup(entity).copied() {
            popup.lifetime += delta_time;

            if popup.lifetime > 1.5 {
                popups_to_remove.push((entity, popup.text_entity));
                continue;
            }

            game_world.set_money_popup(entity, popup);

            if let Some(transform) = world.get_local_transform_mut(popup.text_entity) {
                transform.translation.y += delta_time * 0.5;
                world.set_local_transform_dirty(popup.text_entity, LocalTransformDirty);
            }

            if let Some(text_component) = world.get_text_mut(popup.text_entity) {
                let alpha = (1.0 - (popup.lifetime / 1.5)).max(0.0);
                text_component.properties.color.w = alpha;
                text_component.dirty = true;
            }
        }
    }

    for (entity, text_entity) in popups_to_remove {
        if world.get_text(text_entity).is_some() {
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive {
                    entity: text_entity,
                });
        }
        game_world.despawn_entities(&[entity]);
    }
}
