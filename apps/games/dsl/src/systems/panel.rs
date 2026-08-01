use crate::data;
use crate::resources::{Script, Switcher};
use nightshade::prelude::*;
use nightshade_ui::prelude::*;

pub fn build(
    script: Res<Script>,
    mut switcher: ResMut<Switcher>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let names = data::games();
    let fault = script.fault.clone();
    let buttons = {
        let mut tree = UiTreeBuilder::new(build.context());

        let docked = tree.add_docked_panel_left("dsl", "Games", data::PANEL_WIDTH);
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

        let mut buttons = Vec::new();
        tree.in_parent(content, |tree| {
            tree.add_heading("Description files");
            for name in &names {
                buttons.push(((*name).to_string(), tree.add_button(name)));
            }
            tree.add_separator();
            tree.add_label("Each one is a game");
            tree.add_label("written as a description");
            tree.add_label("and played by this app.");
            if let Some(fault) = &fault {
                tree.add_separator();
                for line in fault.as_bytes().chunks(26) {
                    tree.add_label(&String::from_utf8_lossy(line));
                }
            }
        });

        tree.runtime_mut().visible = true;
        tree.finish();
        buttons
    };

    switcher.buttons = buttons;
}
