use crate::resources::Autosave;
use nightshade::World;
use nightshade::app::{App, Stage, push_frame_system};
use nightshade::plugin::Plugin;

pub struct SavePlugin {
    period: f32,
}

impl SavePlugin {
    pub fn every(period: f32) -> Self {
        Self { period }
    }
}

impl Plugin for SavePlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_platform::contract::window::Window => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        install(app, self.period);
        register_frame_systems(&mut app.stages);
    }
}

fn install(app: &mut App, period: f32) {
    app.insert_resource(Autosave::every(period));
}

fn register_frame_systems(stages: &mut nightshade::ecs::Stages<World>) {
    push_frame_system(
        stages,
        Stage::Update,
        crate::systems::frame::announce_a_due_save,
    );
}
