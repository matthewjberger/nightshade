use crate::ecs::MoneyPopup;
use nightshade::prelude::*;

pub fn spawn_money_popup(world: &mut World, position: Vec3, amount: i32) {
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

    let text_index = world
        .expect_resource_mut::<TextState>()
        .cache
        .add_text(&text);
    let text_entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | TEXT | VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.get_mut::<nightshade::ecs::primitives::Name>(text_entity) {
        *name = Name("Money Popup".to_string());
    }

    if let Some(transform) =
        world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(text_entity)
    {
        transform.translation = position + nalgebra_glm::vec3(0.0, 1.5, 0.0);
    }

    if let Some(text_component) =
        world.get_mut::<nightshade::ecs::text::components::Text>(text_entity)
    {
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

    world.set(text_entity, MoneyPopup { lifetime: 0.0 });
}

pub fn update_money_popups(world: &mut World, delta_time: f32) {
    let mut popups_to_remove: Vec<Entity> = Vec::new();
    let mut fades: Vec<(Entity, f32, f32)> = Vec::new();

    world
        .ecs
        .query_join::<(
            &mut MoneyPopup,
            &nightshade::ecs::transform::components::LocalTransform,
        )>()
        .for_each(|text_entity, (popup, transform)| {
            popup.lifetime += delta_time;
            if popup.lifetime > 1.5 {
                popups_to_remove.push(text_entity);
                return;
            }
            let risen = transform.translation.y + delta_time * 0.5;
            let alpha = (1.0 - (popup.lifetime / 1.5)).max(0.0);
            fades.push((text_entity, risen, alpha));
        });

    for (text_entity, risen, alpha) in fades {
        if let Some(transform) =
            world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(text_entity)
        {
            transform.translation.y = risen;
        }

        if let Some(text_component) =
            world.get_mut::<nightshade::ecs::text::components::Text>(text_entity)
        {
            text_component.properties.color.w = alpha;
            text_component.dirty = true;
        }
    }

    for text_entity in popups_to_remove {
        despawn_recursive_immediate(world, text_entity);
    }
}
