use crate::commands::resources::compose;
use crate::data::systems::{UiSystemFn, UiSystemPhase};
use crate::resources::RetainedUiSchedule;
use crate::systems::schedule::{run_retained_ui_schedule, ui_systems_with};
use nightshade::World;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub fn install(
    resources: &mut nightshade::ecs::ResourceMap,
    systems: Vec<(UiSystemPhase, UiSystemFn)>,
) {
    resources
        .res_mut::<crate::resources::RetainedUiRuntime>()
        .visible = true;
    *resources
        .res::<RetainedUiSchedule>()
        .systems
        .lock()
        .expect("retained ui schedule") = ui_systems_with(systems);
}

pub fn register_frame_systems(stages: &mut nightshade::ecs::Stages<World>) {
    nightshade::app::push_frame_system(stages, Stage::FramePostUpdate, run_retained_ui_schedule);
    stages.stage_mut(Stage::FramePostUpdate.name()).order(
        std::any::type_name_of_val(&nightshade_scene::systems::update_global_transforms_system),
        std::any::type_name_of_val(&run_retained_ui_schedule),
    );
    nightshade::app::push_frame_system(
        stages,
        Stage::Last,
        crate::systems::render_sync::publish_frame,
    );
}

#[derive(Default)]
pub struct UiPlugin {
    systems: Vec<(UiSystemPhase, UiSystemFn)>,
}

impl UiPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_system(mut self, phase: UiSystemPhase, system: UiSystemFn) -> Self {
        self.systems.push((phase, system));
        self
    }
}

impl Plugin for UiPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_platform::contract::input::Input => "PlatformPlugin",
            nightshade_platform::contract::input_capture::InputCapture => "PlatformPlugin",
            nightshade_platform::contract::window::Window => "PlatformPlugin",
            nightshade_input::contract::Actions<nightshade_input::contract::EngineAction> => "InputPlugin<EngineAction>",
        ]
    }
    fn build(&self, app: &mut App) {
        app.add_plugin(nightshade_text::plugin::TextPlugin);
        compose(&mut app.world.resources);
        install(&mut app.world.resources, Vec::new());
        register_frame_systems(&mut app.stages);
    }
}
