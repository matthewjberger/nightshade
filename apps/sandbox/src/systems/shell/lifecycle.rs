use crate::systems::shell::commands;
use crate::systems::shell::state::{SandboxShellContext, ShellAction};
use nightshade::prelude::*;
use nightshade::shell::{ShellState, shell_retained_ui};

pub fn new_shell() -> ShellState<SandboxShellContext> {
    let mut shell = ShellState::new(SandboxShellContext::default());
    shell.height = 420.0;
    shell.register_builtin_commands();
    commands::register(&mut shell);
    shell
}

pub fn run(shell: &mut ShellState<SandboxShellContext>, world: &mut World) {
    if shell.visible {
        let alt_held = world
            .res::<Input>()
            .keyboard
            .is_key_pressed(KeyCode::AltLeft)
            || world
                .res::<Input>()
                .keyboard
                .is_key_pressed(KeyCode::AltRight);
        if !alt_held {
            for character in world.res::<Input>().keyboard.frame_chars.clone() {
                if !character.is_control() {
                    shell.input_buffer.push(character);
                }
            }
        }
        handle_pick_click(shell, world);
    }
    let delta_time = world.res::<Time>().delta_time;
    shell.update_animation(delta_time);
    shell_retained_ui(shell, world);
}

fn handle_pick_click(shell: &mut ShellState<SandboxShellContext>, world: &mut World) {
    let mouse = &world.res::<Input>().mouse;
    if !mouse
        .state
        .contains(nightshade::ecs::input::resources::MouseState::LEFT_JUST_PRESSED)
    {
        return;
    }
    let screen_position = nightshade::ecs::input::access::mouse_for_active(world).position;
    let Some(result) =
        nightshade::ecs::picking::queries::pick_closest_entity(world, screen_position)
    else {
        return;
    };
    shell.insert_entity(result.entity);
}

pub fn handle_key(
    shell: &mut ShellState<SandboxShellContext>,
    world: &mut World,
    key_code: KeyCode,
    key_state: ElementState,
) {
    let pressed = key_state == ElementState::Pressed;
    if pressed {
        let alt_pressed = world
            .res::<Input>()
            .keyboard
            .is_key_pressed(KeyCode::AltLeft)
            || world
                .res::<Input>()
                .keyboard
                .is_key_pressed(KeyCode::AltRight);
        if (key_code == KeyCode::KeyC && alt_pressed) || key_code == KeyCode::Backquote {
            shell.toggle();
            return;
        }
    }
    if shell.visible {
        shell.handle_key(key_code, pressed);
    }
}

pub fn is_capturing_input(shell: &ShellState<SandboxShellContext>) -> bool {
    shell.visible || shell.dragging_resize || shell.dragging_scrollbar
}

pub fn drain_queue(shell: &mut ShellState<SandboxShellContext>) -> Vec<ShellAction> {
    std::mem::take(&mut shell.context.queue)
}
