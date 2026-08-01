use crate::data::Ask;
use crate::resources::Screen;
use crate::systems::shell::SandboxShellContext;
use crate::systems::world::player;
use nightshade::prelude::*;
use nightshade_shell::resources::state::ShellState;
use nightshade_state::prelude::*;

pub fn handle_global(
    mut sandbox_world: ResMut<crate::resources::SandboxResources>,
    shell_state: Option<Res<ShellState<SandboxShellContext>>>,
    screen_state: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
    stage: crate::commands::Stage,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    if shell_state
        .as_deref()
        .is_some_and(nightshade_shell::shell_is_capturing_input)
    {
        return;
    }
    let mut owned = crate::commands::Scene {
        storage,
        commands: &mut commands,
        stage,
    };
    let scene = &mut owned;
    let actions = scene.stage.actions;
    let quit = actions.just_pressed(Ask::Quit);
    let pause = actions.just_pressed(Ask::Pause);
    let reset = actions.just_pressed(Ask::Reset);

    match screen_state.current {
        Screen::Title => {
            if quit {
                scene.stage.window.should_exit = true;
            }
        }
        Screen::InGame => {
            if pause {
                next.pending = Some(Screen::Paused);
            } else if reset {
                player::reset(&mut sandbox_world, scene);
            }
        }
        Screen::Paused => {
            if pause {
                next.pending = Some(Screen::InGame);
            }
        }
    }
}
