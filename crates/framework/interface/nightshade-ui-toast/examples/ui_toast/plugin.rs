use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::resources::Sign::default());
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_plugin(nightshade_toast::ToastPlugin {
            settings: nightshade_toast::ToastSettings {
                showing: theme::SHOWING,
                fading: theme::FADING,
            },
        });
        app.add_plugin(nightshade_ui::UiPlugin::new());
        app.add_plugin(nightshade_ui_toast::UiToastPlugin::default());
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::Startup, build_the_card);
        app.add_system(Stage::First, systems::play::take_the_orders);
        app.add_system(Stage::Update, systems::play::watch_the_card);
    }
}

fn build_the_card(
    style: nightshade::ecs::system_param::Res<nightshade_ui_toast::ToastStyle>,
    mut card: nightshade::ecs::system_param::ResMut<nightshade_ui_toast::ToastCard>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    nightshade_ui_toast::commands::build(&mut tree, &style, &mut card);
    tree.finish();
}
