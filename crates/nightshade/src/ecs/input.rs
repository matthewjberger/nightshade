//! Input handling for keyboard, mouse, gamepad, and touch.
//!
//! Input state is collected from winit events and exposed through the [`Input`] resource:
//!
//! - [`Keyboard`]: Key press states and per-frame character input
//! - [`Mouse`]: Button states, position, deltas, and scroll wheel
//! - [`Gamepad`]: Controller state via gilrs (requires `gamepad` feature)
//! - [`Touch`]: Multi-touch points with gesture detection
//!
//! Query input state in your update functions via the `Input` resource.
//!
//! # Keyboard Input
//!
//! Check if keys are currently held:
//!
//! ```ignore
//! let keyboard = &world.resources.input.keyboard;
//!
//! if keyboard.is_key_pressed(KeyCode::KeyW) {
//!     // W is held down
//! }
//!
//! // Movement vector from WASD
//! let forward = keyboard.is_key_pressed(KeyCode::KeyW) as i32
//!             - keyboard.is_key_pressed(KeyCode::KeyS) as i32;
//! let right = keyboard.is_key_pressed(KeyCode::KeyD) as i32
//!           - keyboard.is_key_pressed(KeyCode::KeyA) as i32;
//! ```
//!
//! For key press/release events, use the [`State::on_keyboard_input`] callback:
//!
//! ```ignore
//! fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
//!     if state == KeyState::Pressed && key == KeyCode::Space {
//!         // Space was just pressed
//!     }
//! }
//! ```
//!
//! # Mouse Input
//!
//! ```ignore
//! let mouse = &world.resources.input.mouse;
//!
//! // Current cursor position in screen coordinates
//! let position = mouse.position;
//!
//! // Movement since last frame (for FPS camera look)
//! let delta = mouse.raw_mouse_delta;
//!
//! // Scroll wheel delta (y = vertical scroll)
//! let scroll = mouse.wheel_delta.y;
//!
//! // Button state flags
//! if mouse.state.contains(MouseState::LEFT_CLICKED) {
//!     // Left button is held
//! }
//! if mouse.state.contains(MouseState::LEFT_JUST_PRESSED) {
//!     // Left button was pressed this frame
//! }
//! if mouse.state.contains(MouseState::RIGHT_JUST_RELEASED) {
//!     // Right button was released this frame
//! }
//! ```
//!
//! # Gamepad Input
//!
//! Requires the `gamepad` feature. Query the active gamepad:
//!
//! ```ignore
//! use gilrs::{Axis, Button};
//!
//! if let Some(gamepad) = query_active_gamepad(world) {
//!     // Analog stick values (-1.0 to 1.0)
//!     let left_x = gamepad.axis_data(Axis::LeftStickX)
//!         .map(|d| d.value()).unwrap_or(0.0);
//!     let left_y = gamepad.axis_data(Axis::LeftStickY)
//!         .map(|d| d.value()).unwrap_or(0.0);
//!
//!     // Button states
//!     if gamepad.is_pressed(Button::South) {  // A button on Xbox
//!         // Jump
//!     }
//!     if gamepad.is_pressed(Button::Start) {
//!         // Pause
//!     }
//! }
//! ```
//!
//! # File Drops
//!
//! Handle files dragged onto the window via [`State::on_dropped_file_data`]:
//!
//! ```ignore
//! fn on_dropped_file_data(&mut self, world: &mut World, name: &str, data: &[u8]) {
//!     if name.ends_with(".glb") || name.ends_with(".gltf") {
//!         // Load the model from data bytes
//!     }
//! }
//! ```
//!
//! [`State::on_keyboard_input`]: crate::run::State::on_keyboard_input
//! [`State::on_dropped_file_data`]: crate::run::State::on_dropped_file_data

pub mod access;
pub mod components;
pub mod events;
pub mod queries;
pub mod resources;
pub mod systems;

pub use access::*;
pub use components::*;
pub use events::*;
#[cfg(feature = "gamepad")]
pub use queries::*;
pub use resources::*;
pub use systems::*;
