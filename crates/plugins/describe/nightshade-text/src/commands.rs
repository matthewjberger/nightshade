use crate::contract::Text;
use crate::contract::TextProperties;
use nalgebra_glm::Vec2;
use nightshade::ecs::Entity;
use nightshade::ecs::system_param::Commands;

pub fn spawn_ui_text(
    text_state: &mut crate::contract::TextState,
    commands: &mut Commands<'_>,
    text: impl Into<String>,
    _position: Vec2,
) -> Entity {
    let text_index = text_state.cache.add_text(text);
    commands.spawn((Text::new(text_index),))
}

pub fn spawn_ui_text_with_properties(
    text_state: &mut crate::contract::TextState,
    commands: &mut Commands<'_>,
    text: impl Into<String>,
    _position: Vec2,
    properties: TextProperties,
) -> Entity {
    let text_index = text_state.cache.add_text(text);
    commands.spawn((Text::new(text_index).with_properties(properties),))
}

pub fn spawn_3d_text_with_properties(
    text_state: &mut crate::contract::TextState,
    commands: &mut Commands<'_>,
    text: &str,
    position: nalgebra_glm::Vec3,
    properties: TextProperties,
) -> Entity {
    spawn_3d_text(text_state, commands, text, position, properties, false)
}

pub fn spawn_3d_billboard_text_with_properties(
    text_state: &mut crate::contract::TextState,
    commands: &mut Commands<'_>,
    text: &str,
    position: nalgebra_glm::Vec3,
    properties: TextProperties,
) -> Entity {
    spawn_3d_text(text_state, commands, text, position, properties, true)
}

fn spawn_3d_text(
    text_state: &mut crate::contract::TextState,
    commands: &mut Commands<'_>,
    text: &str,
    position: nalgebra_glm::Vec3,
    properties: TextProperties,
    billboard: bool,
) -> Entity {
    let text_index = text_state.cache.add_text(text);

    commands.spawn((
        nightshade_scene::contract::primitives::Name(text.to_string()),
        nightshade_scene::contract::LocalTransform {
            translation: position,
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
        },
        nightshade_scene::contract::GlobalTransform::default(),
        Text {
            text_index,
            properties,
            dirty: true,
            cached_mesh: None,
            billboard,
        },
        nightshade_scene::contract::primitives::Visibility { visible: true },
    ))
}
