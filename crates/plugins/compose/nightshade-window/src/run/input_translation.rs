use nightshade::ecs::{EventBus, ResourceMap};
use winit::event::WindowEvent;

fn emit(
    events: &mut EventBus,
    resources: &mut ResourceMap,
    event: nightshade_platform::contract::AppEvent,
) {
    nightshade_platform::commands::emit_app_event(
        events,
        resources.res_mut::<nightshade_platform::contract::input::Input>(),
        event,
    );
}

pub(crate) fn forward_window_events(
    events: &mut EventBus,
    resources: &mut ResourceMap,
    event: &WindowEvent,
) {
    use nightshade_platform::contract::AppEvent;

    match event {
        WindowEvent::DroppedFile(path) => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if path.is_dir() {
                    let mut files = Vec::new();
                    super::file_drop::collect_directory_files(path, path, &mut files);
                    if files.is_empty() {
                        emit(events, resources, AppEvent::FileDroppedPath(path.clone()));
                    } else {
                        for file in files {
                            emit(events, resources, AppEvent::FileDropped(file));
                        }
                    }
                } else {
                    emit(events, resources, AppEvent::FileDroppedPath(path.clone()));
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                emit(events, resources, AppEvent::FileDroppedPath(path.clone()));
            }
        }
        WindowEvent::HoveredFile(path) => {
            emit(events, resources, AppEvent::FileHovered(path.clone()));
        }
        WindowEvent::HoveredFileCancelled => {
            emit(events, resources, AppEvent::FileHoverCancelled);
        }
        _ => {}
    }

    if resources
        .res::<nightshade_platform::contract::input_capture::InputCapture>()
        .event_consumed
    {
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
        emit(
            events,
            resources,
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
        emit(
            events,
            resources,
            AppEvent::Mouse {
                button: *button,
                state: *mouse_state,
            },
        );
    }
}
