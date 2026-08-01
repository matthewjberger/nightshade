use nightshade::World;
use nightshade::app::{App, Stage, push_frame_system};
use nightshade::plugin::Plugin;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        install(&mut app.world.resources);
        register_frame_systems(&mut app.stages);
    }
}

pub fn register_frame_systems(stages: &mut nightshade::ecs::Stages<World>) {
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::systems::emitters::update_particle_emitters,
    );
    push_frame_system(
        stages,
        Stage::Render,
        crate::systems::frame::gather_emitters_system,
    );
}

pub fn install(resources: &mut nightshade::ecs::ResourceMap) {
    resources.insert(crate::contract::ParticleTextures::default());
    resources.insert(crate::contract::ParticleFrame::default());
}
