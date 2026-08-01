use crate::resources::DecalsResources;
use crate::systems::decals::{spawn_decal_at, spawn_random_decals};
use crate::theme::RANDOM_DECAL_COUNT;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::ui_set_visible;
use nightshade_ui::resources::RetainedUiDirty;

pub fn handle_events(
    mut decals: ResMut<DecalsResources>,
    reader: EventReader<AppEvent>,
    capture: Res<InputCapture>,
    mut window: ResMut<Window>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    let decals = &mut *decals;
    for event in reader.iter() {
        match event {
            AppEvent::Keyboard {
                key,
                state: KeyState::Pressed,
            } => match key {
                KeyCode::Escape | KeyCode::KeyQ => window.should_exit = true,
                KeyCode::Tab => decals.current_decal_type = decals.current_decal_type.next(),
                KeyCode::KeyR => {
                    spawn_random_decals(&mut commands, RANDOM_DECAL_COUNT, decals.emissive_strength)
                }
                KeyCode::KeyC => decals.clear_requested = true,
                KeyCode::KeyH => {
                    decals.show_controls = !decals.show_controls;
                    ui_set_visible(
                        &mut dirty,
                        &mut commands,
                        decals.handles.panel,
                        decals.show_controls,
                    );
                }
                _ => {}
            },
            AppEvent::Mouse {
                button: MouseButton::Left,
                state: KeyState::Pressed,
            } => {
                if pointer_captured(&capture) {
                    continue;
                }
                if let Some(pick) = &decals.last_pick_result {
                    spawn_decal_at(
                        &mut commands,
                        pick.world_position,
                        pick.world_normal,
                        decals.current_decal_type,
                        decals.decal_size,
                        decals.emissive_strength,
                        decals.use_awesomeface,
                    );
                }
            }
            _ => {}
        }
    }
}
