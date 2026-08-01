use crate::data;
use crate::resources::{Clock, Tape, Transport};
use nightshade::prelude::*;
use nightshade_text::contract::resources::TextState;
use nightshade_ui::components::layout_types::UiLayoutType;
use nightshade_ui::prelude::*;

pub fn build(
    mut transport: ResMut<Transport>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let made = {
        let mut tree = UiTreeBuilder::new(build.context());

        let docked = tree.add_docked_panel_bottom("transport", "Timeline", data::TRANSPORT_HEIGHT);
        let content = tree
            .component::<UiPanelData>(docked)
            .map(|data| data.content_entity)
            .expect("the panel has a content node");
        tree.edit::<UiLayoutNode>(content, |node| {
            node.flow_layout = Some(FlowLayout {
                direction: FlowDirection::Vertical,
                padding: data::PANEL_PADDING,
                spacing: data::PANEL_SPACING,
                alignment: FlowAlignment::Start,
                cross_alignment: FlowAlignment::Start,
                wrap: false,
            });
        });

        let mut held = Transport::default();
        tree.in_parent(content, |tree| {
            let row = tree.add_row();
            let keys = tree.in_parent(row, |tree| {
                [
                    tree.add_icon_button(icons::material::REFRESH, data::ICON_SIZE),
                    tree.add_icon_button(icons::material::CHEVRON_LEFT, data::ICON_SIZE),
                    tree.add_icon_button(icons::material::PLAY_ARROW, data::ICON_SIZE),
                    tree.add_icon_button(icons::material::PAUSE, data::ICON_SIZE),
                    tree.add_icon_button(icons::material::ARROW_FORWARD, data::ICON_SIZE),
                    tree.add_icon_button(icons::material::CHEVRON_RIGHT, data::ICON_SIZE),
                ]
            });
            for key in keys {
                tree.edit::<UiLayoutNode>(key, |node| {
                    node.flow_child_size =
                        Some(Ab(Vec2::new(data::BUTTON_WIDTH, data::BUTTON_HEIGHT)).into());
                });
            }
            held.restart = Some(keys[0]);
            held.back = Some(keys[1]);
            held.play = Some(keys[2]);
            held.pause = Some(keys[3]);
            held.step = Some(keys[4]);
            held.ahead = Some(keys[5]);
            held.scrub = Some(tree.add_slider(0.0, 1.0, 1.0));
        });

        tree.runtime_mut().visible = true;
        tree.finish();
        held
    };

    *transport = made;
}

pub fn follow(
    tape: Res<Tape>,
    clock: Res<Clock>,
    transport: Res<Transport>,
    mut text: ResMut<TextState>,
    mut sliders: Query<(&mut UiSliderData,)>,
    mut nodes: Query<(&mut UiLayoutNode,)>,
) {
    let Some(scrub) = transport.scrub else {
        return;
    };
    let last = tape.frames.len().saturating_sub(1);
    let at = tape.at();
    let part = if last == 0 {
        0.0
    } else {
        at as f32 / last as f32
    };

    let mut fill = None;
    let mut slot = None;
    sliders.for_entity(scrub, |(data,)| {
        data.value = part;
        fill = Some(data.fill_entity);
        slot = Some(data.text_slot);
    });

    if let Some(fill) = fill {
        nodes.for_entity(fill, |(node,)| {
            if let Some(UiLayoutType::Boundary(boundary)) = node.base_layout.as_mut() {
                boundary.position_2 = Rl(Vec2::new(part * 100.0, 100.0)).into();
            }
        });
    }

    if let Some(slot) = slot {
        let state = if clock.paused { "held" } else { "running" };
        text.cache
            .set_text(slot, format!("turn {at} of {last}, {state}"));
    }
}
