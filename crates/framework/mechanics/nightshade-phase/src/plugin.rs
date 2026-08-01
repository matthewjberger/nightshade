use crate::data::PhaseLook;
use crate::resources::{Phase, PhaseSettings};
use crate::systems::frame::settle_the_phase;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct PhasePlugin {
    pub looks: Vec<PhaseLook>,
    pub start: usize,
}

impl Plugin for PhasePlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_render::data::config::RenderSettings => "GraphicsPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(PhaseSettings {
            looks: self.looks.clone(),
        });
        app.insert_resource(Phase {
            at: self.start,
            applied: None,
        });
        app.add_system(Stage::LateUpdate, settle_the_phase);
    }
}
