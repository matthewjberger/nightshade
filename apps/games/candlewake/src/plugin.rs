use crate::data;
use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_story::prelude::{Play, Story, begin, read};

pub struct CandlewakePlugin;

impl Plugin for CandlewakePlugin {
    fn build(&self, app: &mut App) {
        let (story, play) = initialize();
        app.insert_resource(story);
        app.insert_resource(play);
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_ui_story::UiStoryPlugin {
            style: theme::style(),
        });
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
    }
}

fn initialize() -> (Story, Play) {
    let story = match read(data::SOURCE) {
        Ok(story) => story,
        Err(fault) => panic!("the story does not read: {fault}"),
    };
    let mut play = Play::default();
    begin(&mut play, &story);
    (story, play)
}
