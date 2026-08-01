use crate::resources::PhysicsWorld;

pub fn install(world: &mut nightshade::World) {
    if world.resources.get::<PhysicsWorld>().is_none() {
        world.insert_resource(PhysicsWorld::default());
    }
    world.res_mut::<PhysicsWorld>().enabled = true;
}

pub fn register_frame_systems(stages: &mut nightshade::ecs::Stages<nightshade::World>) {
    use nightshade::app::{Stage, push_frame_system};
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::systems::simulation::run_physics_systems,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::systems::collision_events::emit_collision_events_system,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::systems::camera_sweep::answer_camera_sweep_system,
    );
}

pub struct PhysicsPlugin;
impl nightshade::plugin::Plugin for PhysicsPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_platform::contract::input::Input => "PlatformPlugin",
            nightshade_render::resources::mesh_state::MeshRenderState => "GraphicsPlugin",
            nightshade_render::resources::material_registry::MaterialRegistry => "GraphicsPlugin",
            nightshade_camera::resources::ActiveCamera => "CameraPlugin",
            nightshade_input::contract::Actions<nightshade_input::contract::EngineAction> => "InputPlugin<EngineAction>",
        ]
    }

    fn build(&self, app: &mut nightshade::app::App) {
        app.world
            .insert_resource(crate::resources::PhysicsWorld::default());
        app.world
            .insert_resource(crate::resources::PickingWorld::default());
        install(&mut app.world);
        register_frame_systems(&mut app.stages);
    }
}
