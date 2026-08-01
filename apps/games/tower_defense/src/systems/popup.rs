use crate::commands::Scene;
use crate::components::{Age, Lifetime};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_text::prelude::*;

pub fn spawn_money_popup(scene: &mut Scene<'_, '_>, position: Vec3, amount: i32) {
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

    let text_index = scene.text_state.cache.add_text(&text);
    let text_entity = scene.commands.spawn(());

    scene
        .commands
        .set(text_entity, Name("Money Popup".to_string()));
    scene.commands.set(
        text_entity,
        LocalTransform {
            translation: position + nalgebra_glm::vec3(0.0, 1.5, 0.0),
            ..Default::default()
        },
    );
    scene.commands.set(text_entity, GlobalTransform::default());
    scene.commands.set(
        text_entity,
        nightshade_scene::contract::primitives::Visibility { visible: true },
    );
    scene.commands.set(
        text_entity,
        nightshade_text::contract::Text {
            text_index,
            properties: TextProperties {
                font_size: 36.0,
                color,
                alignment: TextAlignment::Center,
                outline_width: 0.08,
                outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
            dirty: true,
            ..Default::default()
        },
    );

    scene.commands.set(text_entity, Age(0.0));
}

pub fn update_money_popups(scene: &mut Scene<'_, '_>, delta_time: f32) {
    let mut popups_to_remove: Vec<Entity> = Vec::new();
    let mut fades: Vec<(Entity, f32, f32)> = Vec::new();

    scene
        .storage
        .query::<(&mut Age, &nightshade_scene::contract::LocalTransform)>()
        .without::<Lifetime>()
        .for_each(|text_entity, (age, transform)| {
            age.0 += delta_time;
            if age.0 > 1.5 {
                popups_to_remove.push(text_entity);
                return;
            }
            let risen = transform.translation.y + delta_time * 0.5;
            let alpha = (1.0 - (age.0 / 1.5)).max(0.0);
            fades.push((text_entity, risen, alpha));
        });

    for (text_entity, risen, alpha) in fades {
        if let Some(transform) = scene
            .storage
            .get_mut::<nightshade_scene::contract::LocalTransform>(text_entity)
        {
            transform.translation.y = risen;
        }

        if let Some(text_component) = scene
            .storage
            .get_mut::<nightshade_text::contract::Text>(text_entity)
        {
            text_component.properties.color.w = alpha;
            text_component.dirty = true;
        }
    }

    for text_entity in popups_to_remove {
        scene.commands.despawn_recursive(text_entity);
    }
}
