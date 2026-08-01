use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::UiBuildContext;
use crate::data::state::UiStateTrait;
use crate::data::units::Ab;
use nightshade_paint::data::Anchor;

use crate::prelude::*;
pub fn ui_tile_add_pane(
    context: &mut UiBuildContext<'_, '_>,
    container: nightshade::ecs::Entity,
    title: &str,
) -> Option<(TileId, nightshade::ecs::Entity)> {
    let content_entity = spawn_tile_pane_entity(context, container);

    let data = context
        .commands
        .staged_mut::<crate::components::UiTileContainerData>(container)?;

    let pane = TileNode::Pane {
        content_entity,
        title: title.to_string(),
    };
    let pane_id = data.alloc(pane);

    let mut current = data.root;
    let target_tabs = loop {
        match data.get(current) {
            Some(TileNode::Tabs { .. }) => break Some(current),
            Some(TileNode::Split { children, .. }) => current = children[0],
            _ => break None,
        }
    };
    if let Some(tabs_id) = target_tabs
        && let Some(TileNode::Tabs { panes, .. }) = data.get_mut(tabs_id)
    {
        panes.push(pane_id);
    }
    context
        .commands
        .restage::<crate::components::UiTileContainerData>(container);

    Some((pane_id, content_entity))
}

pub fn spawn_tile_pane_entity(
    context: &mut UiBuildContext<'_, '_>,
    container: nightshade::ecs::Entity,
) -> nightshade::ecs::Entity {
    use crate::components::layout_types::{FlowAlignment, FlowDirection, FlowLayout};
    use crate::data::state::UiBase;

    let panel_color = context.theme.active_theme().panel_color;

    let node = crate::components::UiLayoutNode {
        base_layout: Some(crate::components::layout_types::UiLayoutType::Window(
            crate::components::layout_types::WindowLayout {
                position: Ab(Vec2::new(0.0, 0.0)).into(),
                size: Ab(Vec2::new(100.0, 100.0)).into(),
                anchor: Anchor::TopLeft,
            },
        )),
        depth: crate::components::UiDepthMode::Add(0.0),
        clip_content: true,
        flow_layout: Some(FlowLayout {
            direction: FlowDirection::Vertical,
            padding: 4.0,
            spacing: 4.0,
            alignment: FlowAlignment::Start,
            cross_alignment: FlowAlignment::Start,
            wrap: false,
        }),
        ..Default::default()
    };

    let mut color = crate::components::UiNodeColor::default();
    color.colors[UiBase::INDEX] = Some(panel_color);

    let content_entity = context.commands.spawn(());
    context
        .commands
        .set(content_entity, nightshade::ecs::ChildOf(container));
    context.commands.set(content_entity, node);
    context.commands.set(content_entity, color);
    context.commands.set(
        content_entity,
        crate::components::UiNodeContent::Rect {
            corner_radius: 0.0,
            border_width: 0.0,
            border_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
        },
    );
    context
        .commands
        .set(content_entity, crate::components::UiStateWeights::default());

    ui_mark_children_dirty(context.dirty);

    content_entity
}
