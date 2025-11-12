use crate::ecs::event_bus::{EngineEvent, commands::publish_event};
use crate::ecs::world::{Vec2, World, events::*, resources::MouseState};
use crate::plugins::Plugin;
use web_time::Instant;

pub struct UIEventPlugin;

impl Plugin for UIEventPlugin {
    fn on_event(&mut self, world: &mut World, event: &EngineEvent) {
        let EngineEvent::Window(window_event) = event;

        let Some(gui_state) = &mut world.resources.user_interface.state else {
            return;
        };
        let Some(window_handle) = world.resources.window.handle.as_ref() else {
            return;
        };
        world.resources.user_interface.consumed_event = gui_state
            .on_window_event(window_handle, window_event)
            .consumed;
    }

    fn step(&mut self, _world: &mut World) {}
}

pub struct InputEventPlugin;

impl Plugin for InputEventPlugin {
    fn on_event(&mut self, world: &mut World, event: &EngineEvent) {
        let EngineEvent::Window(window_event) = event;

        if let winit::event::WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                    state,
                    ..
                },
            ..
        } = window_event
        {
            *world
                .resources
                .input
                .keyboard
                .keystates
                .entry(*key_code)
                .or_insert(*state) = *state;

            let event_state = match state {
                winit::event::ElementState::Pressed => InputEvent::KeyboardInput {
                    key_code: *key_code as u32,
                    state: KeyState::Pressed,
                },
                winit::event::ElementState::Released => InputEvent::KeyboardInput {
                    key_code: *key_code as u32,
                    state: KeyState::Released,
                },
            };

            publish_event(world, Message::Input { event: event_state });
        }

        let mouse = &mut world.resources.input.mouse;
        match window_event {
            winit::event::WindowEvent::MouseInput { button, state, .. } => {
                let clicked = *state == winit::event::ElementState::Pressed;
                match button {
                    winit::event::MouseButton::Left => {
                        mouse.state.set(MouseState::LEFT_CLICKED, clicked);
                    }
                    winit::event::MouseButton::Middle => {
                        mouse.state.set(MouseState::MIDDLE_CLICKED, clicked);
                    }
                    winit::event::MouseButton::Right => {
                        mouse.state.set(MouseState::RIGHT_CLICKED, clicked);
                    }
                    _ => {}
                }
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let last_position = mouse.position;
                let current_position = Vec2::new(position.x as _, position.y as _);
                mouse.position = current_position;
                mouse.position_delta = current_position - last_position;
                mouse.state.set(MouseState::MOVED, true);
            }
            winit::event::WindowEvent::MouseWheel {
                delta: winit::event::MouseScrollDelta::LineDelta(h_lines, v_lines),
                ..
            } => {
                mouse.wheel_delta = Vec2::new(*h_lines, *v_lines);
                mouse.state.set(MouseState::SCROLLED, true);
            }
            _ => {}
        }
    }

    fn step(&mut self, _world: &mut World) {}
}

pub struct TimingPlugin;

impl Plugin for TimingPlugin {
    fn on_event(&mut self, world: &mut World, event: &EngineEvent) {
        if let EngineEvent::Window(winit::event::WindowEvent::RedrawRequested) = event {
            let now = Instant::now();

            let timing = &mut world.resources.window.timing;

            if timing.initial_frame_start_instant.is_none() {
                timing.initial_frame_start_instant = Some(now);
            }

            timing.raw_delta_time = timing
                .last_frame_start_instant
                .map_or(0.0, |last_frame| (now - last_frame).as_secs_f32());

            timing.delta_time = timing.raw_delta_time * timing.time_speed;

            timing.last_frame_start_instant = Some(now);

            if timing.current_frame_start_instant.is_none() {
                timing.current_frame_start_instant = Some(now);
            }

            if let Some(app_start) = timing.initial_frame_start_instant.as_ref() {
                timing.uptime_milliseconds = (now - *app_start).as_millis() as u64;
            }

            timing.frame_counter += 1;
            if let Some(start) = timing.current_frame_start_instant.as_ref() {
                if (now - *start).as_secs_f32() >= 1.0 {
                    timing.frames_per_second = timing.frame_counter as f32;
                    timing.frame_counter = 0;
                    timing.current_frame_start_instant = Some(now);
                }
            } else {
                timing.current_frame_start_instant = Some(now);
            }
        }
    }

    fn step(&mut self, _world: &mut World) {}
}

pub struct GamepadInputPlugin;

impl Plugin for GamepadInputPlugin {
    fn step(&mut self, world: &mut World) {
        crate::ecs::input::systems::gamepad_input_system(world);
    }
}

pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn step(&mut self, world: &mut World) {
        crate::ecs::transform::systems::run_systems(world);
    }
}

pub struct HudTextSyncPlugin;

impl Plugin for HudTextSyncPlugin {
    fn step(&mut self, world: &mut World) {
        crate::ecs::text::systems::sync_hud_text_system(world);
    }
}

pub struct ResetMousePlugin;

impl Plugin for ResetMousePlugin {
    fn step(&mut self, world: &mut World) {
        crate::ecs::input::systems::reset_mouse_system(world);
    }
}

pub struct CommandProcessingPlugin;

impl Plugin for CommandProcessingPlugin {
    fn step(&mut self, world: &mut World) {
        crate::ecs::world::process_commands_system(world);
    }
}
