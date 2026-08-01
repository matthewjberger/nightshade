use crate::contract::PickingOptions;
use crate::resources::{PickWhen, Picked};
use nightshade::World;
use nightshade::app::{App, Stage, push_frame_system};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct PickingPlugin {
    when: PickWhen,
}

impl PickingPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn when(mut self, when: PickWhen) -> Self {
        self.when = when;
        self
    }
}

impl Plugin for PickingPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::input::Input => "PlatformPlugin",
            nightshade_platform::contract::input_capture::InputCapture => "PlatformPlugin",
            nightshade_platform::contract::window::Window => "PlatformPlugin",
            nightshade_render::resources::viewport::Viewport => "GraphicsPlugin",
            nightshade_camera::resources::ActiveCamera => "CameraPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        install(app, self.when);
        register_frame_systems(&mut app.stages);
    }
}

fn install(app: &mut App, when: PickWhen) {
    app.insert_resource(when);
    app.insert_resource(PickingOptions::default());
    app.insert_resource(Picked::default());
}

fn register_frame_systems(stages: &mut nightshade::ecs::Stages<World>) {
    push_frame_system(
        stages,
        Stage::Update,
        crate::systems::frame::pick_under_the_pointer,
    );
}
