use nightshade::World;
use nightshade::app::{App, Stage, push_frame_system};
use nightshade::plugin::Plugin;

pub struct NavmeshPlugin;

impl Plugin for NavmeshPlugin {
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
    use crate::systems::agents;
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        agents::update_agent_triangles_system,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        agents::process_path_requests_system,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        agents::gather_agent_neighborhood_system,
    );
    push_frame_system(stages, Stage::FrameUpdate, agents::agent_avoidance_system);
    push_frame_system(stages, Stage::FrameUpdate, agents::agent_movement_system);
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        agents::snap_idle_agents_to_navmesh,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::systems::debug::navmesh_debug_draw_system,
    );
}

pub fn install(resources: &mut nightshade::ecs::ResourceMap) {
    if resources.get::<crate::resources::NavMeshWorld>().is_none() {
        resources.insert(crate::resources::NavMeshWorld::default());
    }
    if resources
        .get::<crate::resources::AgentNeighborhood>()
        .is_none()
    {
        resources.insert(crate::resources::AgentNeighborhood::default());
    }
}
