use crate::resources::{Remaining, Running};
use crate::systems::frame::spend_the_time;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct CountdownPlugin;

impl Plugin for CountdownPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(Remaining::default());
        app.insert_resource(Running::default());
        app.add_system(Stage::LateUpdate, spend_the_time);
    }
}
