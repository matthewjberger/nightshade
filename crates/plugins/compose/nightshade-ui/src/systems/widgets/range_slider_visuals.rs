use nalgebra_glm;
use nalgebra_glm::Vec2;

pub(crate) struct RangeSliderVisualUpdate {
    pub fill_entity: nightshade::ecs::Entity,
    pub low_thumb: nightshade::ecs::Entity,
    pub high_thumb: nightshade::ecs::Entity,
    pub text_slot: usize,
    pub precision: usize,
    pub low_normalized: f32,
    pub high_normalized: f32,
    pub low_value: f32,
    pub high_value: f32,
    pub thumb_half: f32,
}

pub(crate) fn update_range_slider_visuals(
    nodes: &mut super::handle_scroll::UiNodes<'_, '_>,
    text_state: &mut nightshade_text::contract::resources::TextState,
    update: &RangeSliderVisualUpdate,
) {
    let RangeSliderVisualUpdate {
        fill_entity,
        low_thumb,
        high_thumb,
        text_slot,
        precision,
        low_normalized,
        high_normalized,
        low_value,
        high_value,
        thumb_half,
    } = *update;
    use crate::components::layout_types::UiLayoutType;

    use crate::data::units::{Ab, Rl};

    nodes.for_entity(fill_entity, |(fill_node,)| {
        if let Some(UiLayoutType::Boundary(boundary)) = fill_node.base_layout.as_mut() {
            boundary.position_1 = Rl(Vec2::new(low_normalized * 100.0, 0.0)).into();
            boundary.position_2 = Rl(Vec2::new(high_normalized * 100.0, 100.0)).into();
        }
    });

    nodes.for_entity(low_thumb, |(low_node,)| {
        if let Some(UiLayoutType::Boundary(boundary)) = low_node.base_layout.as_mut() {
            boundary.position_1 =
                Rl(Vec2::new(low_normalized * 100.0, 0.0)) + Ab(Vec2::new(-thumb_half, 0.0));
            boundary.position_2 =
                Rl(Vec2::new(low_normalized * 100.0, 100.0)) + Ab(Vec2::new(thumb_half, 0.0));
        }
    });

    nodes.for_entity(high_thumb, |(high_node,)| {
        if let Some(UiLayoutType::Boundary(boundary)) = high_node.base_layout.as_mut() {
            boundary.position_1 =
                Rl(Vec2::new(high_normalized * 100.0, 0.0)) + Ab(Vec2::new(-thumb_half, 0.0));
            boundary.position_2 =
                Rl(Vec2::new(high_normalized * 100.0, 100.0)) + Ab(Vec2::new(thumb_half, 0.0));
        }
    });

    text_state.cache.set_text(
        text_slot,
        format!(
            "{:.prec$} - {:.prec$}",
            low_value,
            high_value,
            prec = precision,
        ),
    );
}
