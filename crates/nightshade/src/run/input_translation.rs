//! The engine half of input handling. The winit-event decoding lives in
//! `nightshade_platform::input::translation`; here we publish the keyboard
//! event it returns onto the engine event bus and forward app-level events
//! (file drops, and keyboard/mouse events the UI did not consume).

use crate::ecs::world::World;
use winit::event::WindowEvent;

pub(crate) fn input_event_system(world: &mut World, window_event: &WindowEvent) {
    let _span = tracing::info_span!("input_event").entered();
    use crate::ecs::event_bus::commands::publish_event;
    use crate::ecs::world::events::Message;

    if let Some(event) =
        nightshade_platform::input::translation::apply_window_event(&mut world.ecs, window_event)
    {
        publish_event(world, Message::Input { event });
    }
}

pub(crate) fn forward_window_events(world: &mut World, event: &WindowEvent) {
    use crate::platform::input::events::AppEvent;

    match event {
        WindowEvent::DroppedFile(path) => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if path.is_dir() {
                    let mut files = Vec::new();
                    super::file_drop::collect_directory_files(path, path, &mut files);
                    if files.is_empty() {
                        crate::ecs::events::emit_app_event(
                            world,
                            AppEvent::FileDroppedPath(path.clone()),
                        );
                    } else {
                        for file in files {
                            crate::ecs::events::emit_app_event(world, AppEvent::FileDropped(file));
                        }
                    }
                } else {
                    crate::ecs::events::emit_app_event(
                        world,
                        AppEvent::FileDroppedPath(path.clone()),
                    );
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                crate::ecs::events::emit_app_event(world, AppEvent::FileDroppedPath(path.clone()));
            }
        }
        WindowEvent::HoveredFile(path) => {
            crate::ecs::events::emit_app_event(world, AppEvent::FileHovered(path.clone()));
        }
        WindowEvent::HoveredFileCancelled => {
            crate::ecs::events::emit_app_event(world, AppEvent::FileHoverCancelled);
        }
        _ => {}
    }

    if world.res::<crate::ecs::ui::UserInterface>().consumed_event {
        return;
    }

    if let WindowEvent::KeyboardInput {
        event:
            winit::event::KeyEvent {
                physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                state: key_state,
                ..
            },
        ..
    } = event
    {
        crate::ecs::events::emit_app_event(
            world,
            AppEvent::Keyboard {
                key: *key_code,
                state: *key_state,
            },
        );
    }

    if let WindowEvent::MouseInput {
        button,
        state: mouse_state,
        ..
    } = event
    {
        crate::ecs::events::emit_app_event(
            world,
            AppEvent::Mouse {
                button: *button,
                state: *mouse_state,
            },
        );
    }
}
