use crate::components::Spin;
use crate::contract::{TemplateCommands, TemplateEvent};
use crate::data::Ask;
use crate::resources::TemplateResources;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn tick(
    mut template_resources: ResMut<TemplateResources>,
    mut template_commands: ResMut<TemplateCommands>,
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    mut window: ResMut<Window>,
    events: EventReader<TemplateEvent>,
    spinners: Query<(&Spin, &mut LocalTransform)>,
) {
    let example = &mut template_resources.example;
    example.frame_count = example.frame_count.saturating_add(1);
    example.elapsed_seconds += time.delta_time;

    let delta_time = time.delta_time;
    spinners.for_each(|_entity, (spin, transform)| {
        let rotation = nalgebra_glm::quat_angle_axis(spin.speed * delta_time, &Vec3::y());
        transform.rotation = rotation * transform.rotation;
    });

    if actions.just_pressed(Ask::Ping) {
        crate::commands::send_ping(
            &mut template_commands,
            format!("frame {}", example.frame_count),
        );
    }

    for event in &events {
        match event {
            TemplateEvent::Pong { .. } => example.pong_count += 1,
        }
    }

    if actions.just_pressed(Ask::Quit) {
        window.should_exit = true;
    }
}
