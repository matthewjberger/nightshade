use crate::resources::{Noises, SensesSettings};
use crate::systems::frame::{clear_the_noises, notice_the_hiders};
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct SensesPlugin {
    pub settings: SensesSettings,
}

impl Plugin for SensesPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(SensesSettings {
            plane: self.settings.plane,
            crouch_scale: self.settings.crouch_scale,
            moving_scale: self.settings.moving_scale,
            half_angle: self.settings.half_angle,
            noise_gain: self.settings.noise_gain,
            calm: self.settings.calm,
            alarmed_above: self.settings.alarmed_above,
        });
        app.insert_resource(Noises::default());
        app.add_system(Stage::LateUpdate, notice_the_hiders);
        app.add_system(Stage::Last, clear_the_noises);
    }
}
