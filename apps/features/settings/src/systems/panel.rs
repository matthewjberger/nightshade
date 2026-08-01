use crate::data::{PANEL_PADDING, PANEL_SPACING, PANEL_WIDTH, Settings};
use crate::resources::SettingsResources;
use nightshade::prelude::*;
use nightshade_config::contract::Config;
use nightshade_ui::prelude::*;
use nightshade_ui_config::prelude::{ConfigPanel, build_config_rows};

pub fn build(
    context: UiBuildContext<'_, '_>,
    config: &Config<Settings>,
) -> (Entity, ConfigPanel<Settings>) {
    let mut tree = UiTreeBuilder::new(context);
    let panel = tree.add_docked_panel_left("settings", "Settings", PANEL_WIDTH);
    let content = tree.panel_content(panel).expect("panel content");
    tree.edit::<UiLayoutNode>(content, |node| {
        node.flow_layout = Some(FlowLayout {
            direction: FlowDirection::Vertical,
            padding: PANEL_PADDING,
            spacing: PANEL_SPACING,
            alignment: FlowAlignment::Start,
            cross_alignment: FlowAlignment::Start,
            wrap: false,
        });
    });

    let mut title_label = Entity::default();
    let mut rows = ConfigPanel::<Settings>::default();
    tree.in_parent(content, |tree| {
        title_label = tree.add_heading(config.text(Settings::Title));
        rows = build_config_rows(tree, config);
    });
    tree.finish();

    (title_label, rows)
}

pub fn show_the_title(
    config: Res<Config<Settings>>,
    mut resources: ResMut<SettingsResources>,
    mut commands: Commands,
) {
    let title = config.text(Settings::Title);
    if resources.shown_title == title {
        return;
    }
    resources.shown_title = title.to_string();
    ui_set_text(&mut commands, resources.title_label, title);
}
