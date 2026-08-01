use crate::resources::state::ShellState;
use nightshade::prelude::{Res, ResMut};
use nightshade_platform::prelude::*;

pub fn shell_input_system<C: Send + Sync + 'static>(
    shell: Option<ResMut<ShellState<C>>>,
    input: Res<Input>,
    time: Res<Time>,
) {
    let Some(mut shell) = shell else {
        return;
    };
    if shell.visible {
        let alt_held = input.keyboard.is_key_pressed(KeyCode::AltLeft)
            || input.keyboard.is_key_pressed(KeyCode::AltRight);
        let typed: Vec<char> = if alt_held {
            Vec::new()
        } else {
            input.keyboard.frame_chars.clone()
        };
        for character in typed {
            if !character.is_control() {
                shell.input_buffer.push(character);
            }
        }
    }
    shell.update_animation(time.delta_time);
}

pub fn shell_key<C: Send + Sync + 'static>(
    shell: &mut ShellState<C>,
    input: &Input,
    key_code: KeyCode,
    key_state: ElementState,
) {
    let pressed = key_state == ElementState::Pressed;
    if pressed {
        let alt_pressed = input.keyboard.is_key_pressed(KeyCode::AltLeft)
            || input.keyboard.is_key_pressed(KeyCode::AltRight);
        if (key_code == KeyCode::KeyC && alt_pressed) || key_code == KeyCode::Backquote {
            shell.toggle();
            return;
        }
    }
    if shell.visible {
        shell.handle_key(key_code, pressed);
    }
}

pub fn shell_is_capturing_input<C: Send + Sync + 'static>(shell: &ShellState<C>) -> bool {
    shell.visible || shell.dragging_resize
}
