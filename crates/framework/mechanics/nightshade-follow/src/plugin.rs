use crate::resources::FollowSettings;
use crate::systems::frame::follow_the_target;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct FollowPlugin {
    pub settings: FollowSettings,
}

impl Plugin for FollowPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(FollowSettings {
            rate: self.settings.rate,
            snap: self.settings.snap,
            aim: self.settings.aim,
        });
        app.add_system(Stage::FrameUpdate, follow_the_target);
    }
}
