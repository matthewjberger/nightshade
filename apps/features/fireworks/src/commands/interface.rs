use crate::data::{SHAPE_BUTTONS, STANDARD_BUTTONS};
use crate::resources::{Interface, Launcher, Toggles};
use nightshade::prelude::*;
use nightshade_ui::components::layout_types::{FlowAlignment, FlowDirection, FlowLayout};
use nightshade_ui::prelude::*;
use std::collections::HashMap;

fn collapsing<F>(tree: &mut UiTreeBuilder, label: &str, default_open: bool, body: F)
where
    F: FnOnce(&mut UiTreeBuilder),
{
    let header = tree.add_collapsing_header(label, default_open);
    let Some(content) = tree
        .component::<UiCollapsingHeaderData>(header)
        .map(|data| data.content_entity)
    else {
        return;
    };
    tree.in_parent(content, body);
}

pub fn build_ui(
    interface: &mut Interface,
    launcher: &Launcher,
    toggles: &Toggles,
    build: &mut UiBuild<'_>,
    bloom_intensity: f32,
) {
    let salvo_mode_initial = launcher.salvo;

    {
        let mut tree = UiTreeBuilder::new(build.context());
        let panel = tree.add_docked_panel_left("fireworks", "Fireworks", 260.0);
        let content = tree
            .component::<nightshade_ui::components::UiPanelData>(panel)
            .map(|data| data.content_entity)
            .expect("panel content missing");
        tree.edit::<nightshade_ui::components::UiLayoutNode>(content, |node| {
            node.flow_layout = Some(FlowLayout {
                direction: FlowDirection::Vertical,
                padding: 0.0,
                spacing: 0.0,
                alignment: FlowAlignment::Start,
                cross_alignment: FlowAlignment::Start,
                wrap: false,
            });
        });

        let mut type_buttons: HashMap<Entity, crate::data::ExplosionType> = HashMap::new();

        tree.in_parent(content, |tree| {
            let scroll = tree.add_scroll_area_fill(8.0, 6.0);
            let scroll_body = tree
                .component::<UiScrollAreaData>(scroll)
                .map(|data| data.content_entity)
                .expect("scroll body");
            tree.in_parent(scroll_body, |tree| {
                interface.auto_launch = tree.add_checkbox("Auto-launch", launcher.auto);
                tree.add_label("Launch interval (s)");
                interface.launch_interval = tree.add_slider(0.1, 2.0, launcher.interval);

                interface.salvo_mode = tree.add_checkbox("Salvo mode", salvo_mode_initial);
                interface.salvo_count_label = tree.add_label("Salvo count");
                interface.salvo_count = tree.add_slider(3.0, 15.0, launcher.salvo_count as f32);

                tree.add_label("Bloom intensity");
                interface.bloom_intensity = tree.add_slider(0.0, 1.0, bloom_intensity);

                interface.show_grid = tree.add_checkbox("Show grid", toggles.grid);

                interface.launch_single = tree.add_button("Launch Single");
                interface.launch_salvo = tree.add_button("Launch Salvo (5)");
                interface.finale = tree.add_button("FINALE!");

                collapsing(tree, "Launch by Type", false, |tree| {
                    tree.add_label("Standard");
                    for (label, explosion_type) in STANDARD_BUTTONS {
                        type_buttons.insert(tree.add_button(label), explosion_type);
                    }

                    tree.add_label("Shapes");
                    for (label, explosion_type) in SHAPE_BUTTONS {
                        type_buttons.insert(tree.add_button(label), explosion_type);
                    }

                    tree.add_label("Special");
                    type_buttons.insert(
                        tree.add_button("Splitting Shell"),
                        crate::data::ExplosionType::SplittingShell,
                    );
                });

                interface.show_status_label = tree.add_label("");
                interface.show_play = tree.add_button("Play Fireworks Show");

                tree.add_label("");
                tree.add_label("Stats");
                interface.shells_label = tree.add_label("Active shells: 0");
                interface.sub_shells_label = tree.add_label("Sub-shells: 0");
                interface.fire_label = tree.add_label("Fire effects: 0");
                interface.sprite_label = tree.add_label("Sprite effects: 0 (0 entities)");
                interface.smoke_label = tree.add_label("Smoke effects: 0");
                interface.emitters_label = tree.add_label("Active emitters: 0");
            });
        });
        tree.finish();

        interface.type_buttons = type_buttons;
    }

    {
        let context = build.context();
        let dirty = context.dirty;
        let commands = context.commands;
        ui_set_visible(
            dirty,
            commands,
            interface.salvo_count_label,
            salvo_mode_initial,
        );
        ui_set_visible(dirty, commands, interface.salvo_count, salvo_mode_initial);
    }

    interface.last_show_active = true;
    interface.last_salvo_mode = !salvo_mode_initial;
    interface.last_stats = None;
}
