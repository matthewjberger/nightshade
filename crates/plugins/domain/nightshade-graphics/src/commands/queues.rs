use crate::data::queues::{EcsCommand, RenderCommand};
use crate::resources::queues::CommandQueues;
#[cfg(test)]
use nightshade_ecs::World;

pub fn queue_ecs_command(
    commands: &mut nightshade_ecs::system_param::Commands<'_>,
    command: EcsCommand,
) {
    commands.queue(move |_, resources| {
        resources
            .res_mut::<crate::resources::queues::CommandQueues>()
            .ecs
            .push(command);
    });
}

pub fn queue_render_command(queues: &mut CommandQueues, command: RenderCommand) {
    queues.render.push(command);
}

pub fn load_hdr_skybox(queues: &mut CommandQueues, hdr_data: Vec<u8>) {
    queue_render_command(queues, RenderCommand::LoadHdrSkybox { hdr_data });
}

pub fn load_hdr_skybox_from_path(queues: &mut CommandQueues, path: std::path::PathBuf) {
    queue_render_command(queues, RenderCommand::LoadHdrSkyboxFromPath { path });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn capture_screenshot(queues: &mut CommandQueues) {
    queue_render_command(
        queues,
        RenderCommand::CaptureScreenshot {
            path: None,
            max_dimension: None,
        },
    );
}

pub fn remake_renderer(queues: &mut CommandQueues) {
    queue_render_command(queues, RenderCommand::RemakeRenderer);
}

pub fn capture_screenshot_to_path(queues: &mut CommandQueues, path: impl Into<std::path::PathBuf>) {
    queue_render_command(
        queues,
        RenderCommand::CaptureScreenshot {
            path: Some(path.into()),
            max_dimension: None,
        },
    );
}

pub fn capture_thumbnail_to_path(
    queues: &mut CommandQueues,
    path: impl Into<std::path::PathBuf>,
    max_dimension: u32,
) {
    queue_render_command(
        queues,
        RenderCommand::CaptureScreenshot {
            path: Some(path.into()),
            max_dimension: Some(max_dimension),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_commands_system_drains_ecs_and_leaves_render() {
        let mut world = World::default();
        world.insert_resource(nightshade_ecs::HierarchyIndex::default());
        world.insert_resource(crate::resources::queues::CommandQueues::default());
        world.insert_resource(
            nightshade_scene::resources::entity_registry::EntityRegistry::default(),
        );
        world.insert_resource(nightshade_render::resources::mesh_cache::MeshCache::default());
        world.insert_resource(nightshade_render::resources::mesh_state::MeshRenderState::default());
        world.insert_resource(nightshade_render::resources::texture_cache::TextureCache::default());
        world.insert_resource(
            nightshade_render::resources::material_registry::MaterialRegistry::default(),
        );
        let entity = world.spawn(());

        world.with_commands(|_world, commands| {
            queue_ecs_command(commands, EcsCommand::DespawnRecursive { entity });
        });
        queue_render_command(
            world.res_mut::<crate::resources::queues::CommandQueues>(),
            RenderCommand::LoadHdrSkybox {
                hdr_data: vec![0u8; 4],
            },
        );

        let mut drain = nightshade_ecs::system_param::IntoSystem::into_runner(
            crate::commands::builtins::process_commands_system,
        );
        drain.run(&mut world);

        assert!(
            world
                .res::<crate::resources::queues::CommandQueues>()
                .ecs
                .is_empty(),
            "process_commands_system should drain the ecs queue"
        );
        assert_eq!(
            world
                .res::<crate::resources::queues::CommandQueues>()
                .render
                .len(),
            1,
            "process_commands_system must not touch the render queue"
        );
        assert!(matches!(
            world
                .res::<crate::resources::queues::CommandQueues>()
                .render[0],
            RenderCommand::LoadHdrSkybox { .. }
        ));
    }
}
