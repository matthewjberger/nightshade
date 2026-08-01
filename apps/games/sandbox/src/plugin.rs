use crate::commands::Scene;
use crate::data::Ask;
use crate::resources::{SandboxResources, Screen};
use crate::systems::shell::{self, SandboxShellContext};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::systems::controllers::{
    first_person_camera_look_system, fly_camera_system, third_person_gamepad_input,
    third_person_mouse_input, third_person_update_transform,
};
use nightshade_gamepad::GamepadAxis;
use nightshade_graphics::prelude::*;
use nightshade_input::{Actions, EngineAction, InputPlugin};
use nightshade_platform::prelude::*;
use nightshade_shell::resources::state::ShellState;
use nightshade_state::prelude::*;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Sandbox".to_string();
        app.insert_resource(SandboxResources::default());
        app.insert_resource(shell::new_shell());
        app.add_plugin(nightshade_shell::ShellOverlayPlugin::<SandboxShellContext>::default());
        app.add_plugin(StatePlugin::new(Screen::Title));
        app.add_plugin(InputPlugin::<Ask>::new());
        for screen in [Screen::Title, Screen::Paused, Screen::InGame] {
            app.on_enter(
                screen,
                move |mut sandbox_world: ResMut<SandboxResources>,
                      stage: crate::commands::Stage,
                      mut commands: Commands,
                      storage: &mut nightshade::ecs::Storage| {
                    let mut scene = Scene {
                        storage,
                        commands: &mut commands,
                        stage,
                    };
                    crate::systems::lifecycle::enter(&mut sandbox_world, &mut scene, screen);
                },
            );
        }
        app.add_system(Stage::Startup, initialize);
        app.add_system(Stage::Startup, crate::systems::lifecycle::build_ui);
        app.add_system(
            Stage::Update,
            (
                crate::systems::input::handle_global,
                crate::systems::screens::title::handle_input,
                crate::systems::screens::pause::handle_input,
                while_in(
                    Screen::InGame,
                    (
                        update_cursor_lock,
                        fly_camera_look,
                        fly_camera_move,
                        clamp_fly_camera,
                    ),
                ),
                while_in(
                    Screen::InGame,
                    (
                        stand_the_character_model,
                        third_person_camera,
                        third_person_look,
                        third_person_stick,
                        third_person_place,
                        animate_character,
                        crate::systems::world::enemies::update,
                        crate::systems::world::infinite::update,
                    ),
                ),
                crate::systems::screens::hud::update,
                run_shell,
            ),
        );
    }
}

fn initialize(
    mut sandbox_world: ResMut<crate::resources::SandboxResources>,
    mut shell: ResMut<ShellState<SandboxShellContext>>,
    stage: crate::commands::Stage,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let mut scene = crate::commands::Scene {
        storage,
        commands: &mut commands,
        stage,
    };
    crate::systems::lifecycle::initialize(&mut sandbox_world, &mut scene);
    let pose_names: Vec<String> = sandbox_world
        .pose
        .keyframes
        .iter()
        .map(|keyframe| keyframe.name.clone())
        .collect();
    shell.context.pose_names = pose_names;
}

fn update_cursor_lock(
    shell_state: Option<Res<ShellState<SandboxShellContext>>>,
    mut window: ResMut<Window>,
) {
    let cursor_should_lock = !shell_state
        .as_deref()
        .is_some_and(nightshade_shell::shell_is_capturing_input);
    if window.cursor_locked != cursor_should_lock {
        set_cursor_locked(&mut window, cursor_should_lock);
        set_cursor_visible(&mut window, !cursor_should_lock);
    }
}

fn animate_character(
    mut sandbox_world: ResMut<crate::resources::SandboxResources>,
    stage: crate::commands::Stage,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let mut scene = crate::commands::Scene {
        storage,
        commands: &mut commands,
        stage,
    };
    crate::systems::world::pose::update(&mut sandbox_world, &mut scene);
    crate::systems::world::pose::apply_softness(&mut sandbox_world, &mut scene);
    crate::systems::world::ik::update(&mut sandbox_world, &mut scene);
}

fn shell_has_input(shell_state: Option<&ShellState<SandboxShellContext>>) -> bool {
    shell_state.is_some_and(nightshade_shell::shell_is_capturing_input)
}

fn fly_camera_look(
    sandbox_world: Res<SandboxResources>,
    shell_state: Option<Res<ShellState<SandboxShellContext>>>,
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    input: Res<Input>,
    actions: Res<Actions<EngineAction>>,
    time: Res<Time>,
    window: Res<Window>,
    cameras: Query<(&Camera, &mut LocalTransform)>,
) {
    if shell_has_input(shell_state.as_deref())
        || (!sandbox_world.player.fly && sandbox_world.character.third_person)
    {
        return;
    }
    first_person_camera_look_system(active_camera, input, actions, time, window, cameras);
}

fn fly_camera_move(
    sandbox_world: Res<SandboxResources>,
    shell_state: Option<Res<ShellState<SandboxShellContext>>>,
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    capture: Res<nightshade_platform::contract::input_capture::InputCapture>,
    actions: Res<Actions<EngineAction>>,
    time: Res<Time>,
    cameras: Query<
        (&mut Camera, &mut LocalTransform),
        Without<nightshade_camera_controller::contract::PanOrbitCamera>,
    >,
) {
    if !sandbox_world.player.fly || shell_has_input(shell_state.as_deref()) {
        return;
    }
    fly_camera_system(active_camera, capture, actions, time, cameras);
}

fn clamp_fly_camera(
    sandbox_world: Res<crate::resources::SandboxResources>,
    shell_state: Option<Res<ShellState<SandboxShellContext>>>,
    stage: crate::commands::Stage,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    if !sandbox_world.player.fly || shell_has_input(shell_state.as_deref()) {
        return;
    }
    let mut scene = crate::commands::Scene {
        storage,
        commands: &mut commands,
        stage,
    };
    crate::systems::world::player::clamp_fly_camera(&sandbox_world, &mut scene);
}

fn stand_the_character_model(
    mut sandbox_world: ResMut<crate::resources::SandboxResources>,
    stage: crate::commands::Stage,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let mut scene = crate::commands::Scene {
        storage,
        commands: &mut commands,
        stage,
    };
    crate::systems::world::character::attach_the_model(&mut sandbox_world, &mut scene);
}

fn third_person_camera(
    mut sandbox_world: ResMut<crate::resources::SandboxResources>,
    shell_state: Option<Res<ShellState<SandboxShellContext>>>,
    stage: crate::commands::Stage,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    if sandbox_world.player.fly
        || !sandbox_world.character.third_person
        || shell_has_input(shell_state.as_deref())
    {
        return;
    }
    let mut scene = crate::commands::Scene {
        storage,
        commands: &mut commands,
        stage,
    };
    crate::systems::world::character::update(&mut sandbox_world, &mut scene);
}

fn apply_console_wishes(
    shell: &mut ShellState<SandboxShellContext>,
    scene: &mut crate::commands::Scene<'_, '_>,
) {
    let wanted = std::mem::take(&mut shell.context.wanted);
    if let Some(settings) = wanted.settings {
        *scene.stage.settings = settings;
    }
    if let Some(debug_draw) = wanted.debug_draw {
        *scene.stage.debug_draw = debug_draw;
    }
    if let Some(renderer_state) = wanted.renderer_state {
        *scene.stage.renderer_state = renderer_state;
    }
    if let Some(window) = wanted.window {
        scene.stage.window.use_fullscreen = window.use_fullscreen;
        scene.stage.window.show_cursor = window.show_cursor;
    }
    if let Some(speed) = wanted.time_speed {
        scene.stage.time.time_speed = speed;
    }
    if wanted.exit {
        scene.stage.window.should_exit = true;
    }
}

fn read_console_state(
    shell: &mut ShellState<SandboxShellContext>,
    scene: &crate::commands::Scene<'_, '_>,
) {
    let readout = &mut shell.context.readout;
    readout.settings = scene.stage.settings.clone();
    readout.debug_draw = scene.stage.debug_draw.clone();
    readout.renderer_state = scene.stage.renderer_state.clone();
    let state = &scene.stage.gamepad;
    readout.gamepad.connected = state.connected;
    readout.gamepad.just_pressed = state
        .just_pressed
        .iter()
        .map(|button| format!("{button:?}"))
        .collect();
    readout.gamepad.held = state
        .held
        .iter()
        .map(|button| format!("{button:?}"))
        .collect();
    readout.gamepad.left_stick = (
        state.axis(GamepadAxis::LeftStickX),
        state.axis(GamepadAxis::LeftStickY),
    );
    readout.gamepad.right_stick = (
        state.axis(GamepadAxis::RightStickX),
        state.axis(GamepadAxis::RightStickY),
    );
    readout.gamepad.triggers = (
        state.axis(GamepadAxis::LeftTrigger),
        state.axis(GamepadAxis::RightTrigger),
    );
    readout.time_speed = scene.stage.time.time_speed;
    readout.frames_per_second = scene.stage.time.frames_per_second;
    readout.delta_time = scene.stage.time.delta_time;
    readout.uptime_milliseconds = scene.stage.time.uptime_milliseconds;
    readout.window.use_fullscreen = scene.stage.window.use_fullscreen;
    readout.window.show_cursor = scene.stage.window.show_cursor;
}

fn run_shell(
    mut sandbox_world: ResMut<crate::resources::SandboxResources>,
    mut shell_state: ResMut<ShellState<SandboxShellContext>>,
    reader: EventReader<AppEvent>,
    stage: crate::commands::Stage,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let events: Vec<AppEvent> = reader.iter().cloned().collect();
    let mut scene = crate::commands::Scene {
        storage,
        commands: &mut commands,
        stage,
    };
    apply_console_wishes(&mut shell_state, &mut scene);
    for action in shell::drain_queue(&mut shell_state) {
        shell::apply_shell_actions(&mut sandbox_world, &mut scene, action);
    }
    read_console_state(&mut shell_state, &scene);

    for event in &events {
        if let AppEvent::Keyboard { key, state } = event {
            nightshade_shell::shell_key(&mut shell_state, scene.stage.input, *key, *state);
        }
    }
}

fn third_person_is_driven(
    sandbox_world: &SandboxResources,
    shell_state: Option<&ShellState<SandboxShellContext>>,
) -> bool {
    sandbox_world.character.third_person
        && !sandbox_world.player.fly
        && sandbox_world.character.body.is_some()
        && sandbox_world.character.camera.is_some()
        && !shell_has_input(shell_state)
}

fn third_person_look(
    sandbox_world: Res<SandboxResources>,
    shell_state: Option<Res<ShellState<SandboxShellContext>>>,
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    capture: Res<nightshade_platform::contract::input_capture::InputCapture>,
    input: Res<Input>,
    window: Res<Window>,
    cameras: Query<(&mut nightshade_camera_controller::contract::ThirdPersonCamera,)>,
) {
    if !third_person_is_driven(&sandbox_world, shell_state.as_deref()) {
        return;
    }
    third_person_mouse_input(active_camera, capture, input, window, cameras);
}

fn third_person_stick(
    sandbox_world: Res<SandboxResources>,
    shell_state: Option<Res<ShellState<SandboxShellContext>>>,
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    actions: Res<Actions<EngineAction>>,
    time: Res<Time>,
    cameras: Query<(&mut nightshade_camera_controller::contract::ThirdPersonCamera,)>,
) {
    if !third_person_is_driven(&sandbox_world, shell_state.as_deref()) {
        return;
    }
    third_person_gamepad_input(active_camera, actions, time, cameras);
}

fn third_person_place(
    sandbox_world: Res<SandboxResources>,
    shell_state: Option<Res<ShellState<SandboxShellContext>>>,
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    time: Res<Time>,
    storage: &mut nightshade::ecs::Storage,
) {
    if !third_person_is_driven(&sandbox_world, shell_state.as_deref()) {
        return;
    }
    third_person_update_transform(active_camera, time, storage);
}
