use crate::resources::Console;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_platform::contract::AppEvent;
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;
use nightshade_shell::ShellState;

pub fn read_the_keys(
    input: Res<Input>,
    mut shell: ResMut<ShellState<Console>>,
    reader: nightshade::ecs::system_param::EventReader<AppEvent>,
) {
    for event in reader.iter() {
        if let AppEvent::Keyboard { key, state } = event {
            nightshade_shell::shell_key(&mut shell, &input, *key, *state);
        }
    }
}

pub fn follow_the_console(
    time: Res<Time>,
    mut shell: ResMut<ShellState<Console>>,
    mut readout: ResMut<Readout>,
    storage: &mut Storage,
) {
    let delta = time.delta_time;
    let (spin, turn, open) = {
        shell.context.turn += delta * shell.context.spin;
        (shell.context.spin, shell.context.turn, shell.visible)
    };

    let bodies: Vec<_> = storage
        .query_ref::<(&LocalTransform,)>()
        .iter()
        .filter(|(_entity, (transform,))| transform.translation.y > 1.0)
        .map(|(entity, _)| entity)
        .collect();
    for body in bodies {
        if let Some(transform) = storage.get_mut::<LocalTransform>(body) {
            transform.rotation = nalgebra_glm::quat_angle_axis(turn, &nalgebra_glm::Vec3::y());
        }
    }

    readout.notes.clear();
    readout.notes.push(format!("spin {spin:.2}"));
    readout.notes.push(
        if open {
            "console open"
        } else {
            "console closed"
        }
        .to_string(),
    );
}
