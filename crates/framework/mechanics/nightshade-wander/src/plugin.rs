use crate::resources::WanderSettings;
use crate::systems::frame::wander;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct WanderPlugin {
    pub settings: WanderSettings,
}

impl Plugin for WanderPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(WanderSettings {
            reach: self.settings.reach,
            speed: self.settings.speed,
            arrival: self.settings.arrival,
            rest_low: self.settings.rest_low,
            rest_high: self.settings.rest_high,
        });
        app.world
            .resource_or_insert_default::<nightshade_random::Random>();
        app.world
            .resource_or_insert_default::<nightshade_plane::PlayPlane>();
        app.add_system(Stage::Update, wander);
    }
}
