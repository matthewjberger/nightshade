use crate::ecs::{SandboxResources, Screen};
use crate::systems::shell::{self, SandboxShellContext};
use nightshade::ecs::camera::systems::{
    first_person_camera_look_system, wasd_keyboard_controls_system,
};
use nightshade::prelude::*;
use nightshade::shell::ShellState;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Sandbox".to_string();
        app.insert_resource(SandboxResources::default());
        app.insert_resource(shell::new_shell());
        app.insert_state(Screen::Title);
        for screen in [Screen::Title, Screen::Paused, Screen::InGame] {
            app.on_enter(
                screen,
                move |sandbox_world: &mut SandboxResources, world: &mut World| {
                    crate::systems::lifecycle::enter(sandbox_world, world, screen);
                },
            );
        }
        app.add_system(Stage::Startup, initialize);
        app.add_systems(
            Stage::Update,
            (
                crate::systems::input::handle_global,
                crate::systems::save::handle_save_load,
                crate::systems::screens::title::handle_input,
                crate::systems::screens::pause::handle_input,
                while_in(Screen::InGame).run((
                    update_cursor_lock,
                    camera_control,
                    crate::systems::world::enemies::update,
                    crate::systems::world::infinite::update,
                    crate::systems::world::grass::update,
                    crate::systems::world::rocks::update,
                )),
                crate::systems::world::pose::update,
                crate::systems::world::pose::apply_softness,
                crate::systems::world::ik::update,
                crate::systems::screens::hud::update,
                run_shell,
            ),
        );
    }
}

fn initialize(
    sandbox_world: &mut SandboxResources,
    mut shell_state: ResMut<ShellState<SandboxShellContext>>,
    world: &mut World,
) {
    crate::systems::lifecycle::initialize(sandbox_world, world);
    shell_state.context.pose_names = sandbox_world
        .pose
        .keyframes
        .iter()
        .map(|keyframe| keyframe.name.clone())
        .collect();
}

fn update_cursor_lock(world: &mut World) {
    let cursor_should_lock = !world
        .ecs
        .resource::<ShellState<SandboxShellContext>>()
        .is_some_and(shell::is_capturing_input);
    if world.res::<Window>().cursor_locked != cursor_should_lock {
        set_cursor_locked(world, cursor_should_lock);
        set_cursor_visible(world, !cursor_should_lock);
    }
}

fn camera_control(sandbox_world: &mut SandboxResources, world: &mut World) {
    if world
        .ecs
        .resource::<ShellState<SandboxShellContext>>()
        .is_some_and(shell::is_capturing_input)
    {
        return;
    }
    if sandbox_world.player.fly {
        first_person_camera_look_system(world);
        wasd_keyboard_controls_system(world);
        crate::systems::world::player::clamp_fly_camera(sandbox_world, world);
    } else if sandbox_world.character.third_person {
        crate::systems::world::character::update(sandbox_world, world);
    } else {
        first_person_camera_look_system(world);
    }
}

fn run_shell(
    sandbox_world: &mut SandboxResources,
    mut shell_state: ResMut<ShellState<SandboxShellContext>>,
    reader: EventReader<AppEvent>,
    world: &mut World,
) {
    shell::run(&mut shell_state, world);
    for action in shell::drain_queue(&mut shell_state) {
        shell::apply_shell_actions(sandbox_world, world, action);
    }

    for event in reader.read() {
        if let AppEvent::Keyboard { key, state } = event {
            shell::handle_key(&mut shell_state, world, *key, *state);
        }
    }
}
