use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_story::prelude::{Play, begin, read};

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        let story = match read(theme::SOURCE) {
            Ok(story) => story,
            Err(fault) => panic!("the story does not read: {fault}"),
        };
        let mut play = Play::default();
        begin(&mut play, &story);
        app.insert_resource(story);
        app.insert_resource(play);
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_plugin(nightshade_ui_story::UiStoryPlugin {
            style: theme::style(),
        });
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::First, systems::play::take_the_keys);
        app.add_system(Stage::Update, systems::play::show_the_card);
    }
}
