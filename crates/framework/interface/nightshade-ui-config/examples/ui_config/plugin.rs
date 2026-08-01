use crate::data::Panel;
use crate::resources::Shown;
use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Shown::default());
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_plugin(nightshade_ui::UiPlugin::new());
        app.add_plugin(nightshade_config::ConfigPlugin::<Panel>::new());
        app.add_plugin(nightshade_ui_config::ConfigUiPlugin::<Panel>::new());
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::Startup, build_the_panel);
        app.add_system(Stage::Update, systems::play::follow_the_settings);
    }
}

fn build_the_panel(
    config: nightshade::ecs::system_param::Res<nightshade_config::contract::Config<Panel>>,
    mut rows: nightshade::ecs::system_param::ResMut<
        nightshade_ui_config::resources::ConfigPanel<Panel>,
    >,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    let panel = tree.add_docked_panel_left("settings", theme::PANEL_TITLE, theme::PANEL_WIDTH);
    let content = tree.panel_content(panel).expect("panel content");
    tree.edit::<nightshade_ui::prelude::UiLayoutNode>(content, |node| {
        node.flow_layout = Some(nightshade_ui::prelude::FlowLayout {
            direction: nightshade_ui::prelude::FlowDirection::Vertical,
            padding: theme::PANEL_PADDING,
            spacing: theme::PANEL_SPACING,
            alignment: nightshade_ui::prelude::FlowAlignment::Start,
            cross_alignment: nightshade_ui::prelude::FlowAlignment::Start,
            wrap: false,
        });
    });
    tree.in_parent(content, |tree| {
        *rows = nightshade_ui_config::commands::build_config_rows(tree, &config);
    });
    tree.finish();
}
