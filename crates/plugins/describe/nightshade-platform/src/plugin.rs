use nightshade::app::{App, Stage, push_frame_system};
use nightshade::plugin::Plugin;

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        install(app);
        register_frame_systems(&mut app.stages);
    }
}

pub fn install(app: &mut App) {
    app.world
        .resource_or_insert_default::<crate::contract::input::Input>();
    app.world
        .resource_or_insert_default::<crate::contract::input::AndroidGamepadEvents>();
    app.world
        .resource_or_insert_default::<crate::contract::input::AndroidKeyboard>();
    app.world
        .resource_or_insert_default::<crate::contract::input_capture::InputCapture>();
    app.world
        .resource_or_insert_default::<crate::contract::time::Time>();
    app.world
        .resource_or_insert_default::<crate::contract::window::Monitors>();
    if app
        .world
        .resources
        .get::<crate::contract::window::Window>()
        .is_none()
    {
        app.world.insert_resource(crate::contract::window::Window {
            start_hidden: true,
            ..Default::default()
        });
    }
}

pub fn register_frame_systems(stages: &mut nightshade::ecs::Stages<nightshade::ecs::World>) {
    #[cfg(target_arch = "wasm32")]
    push_frame_system(
        stages,
        Stage::First,
        crate::systems::sync_cursor_lock_state_system,
    );
    #[cfg(not(target_arch = "wasm32"))]
    push_frame_system(
        stages,
        Stage::First,
        crate::systems::recenter_locked_cursor_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::systems::apply_ime_allowed_system,
    );
    push_frame_system(stages, Stage::Last, crate::systems::reset_mouse_system);
    push_frame_system(stages, Stage::Last, crate::systems::reset_keyboard_system);
    push_frame_system(stages, Stage::Last, crate::systems::reset_touch_system);
    push_frame_system(
        stages,
        Stage::Last,
        crate::systems::reset_android_gamepad_events_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::systems::reset_input_events_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::systems::apply_requested_cursor_system,
    );
    #[cfg(not(target_arch = "wasm32"))]
    push_frame_system(
        stages,
        Stage::Last,
        crate::systems::apply_cursor_state_system,
    );
}
