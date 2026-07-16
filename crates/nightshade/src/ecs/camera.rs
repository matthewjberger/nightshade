//! Camera system for view and projection management.
//!
//! Control how the 3D scene is viewed with cameras and projections:
//!
//! - [`Camera`]: Main camera component with projection settings
//! - [`Projection`]: Perspective or orthographic projection mode
//! - [`PanOrbitCamera`]: Arc-ball style camera controller for editor-like navigation
//! - [`Smoothing`]: Input smoothing parameters for fluid camera movement
//!
//! Both projection types use reverse-Z depth for improved precision at distance.
//!
//! # Creating a Camera
//!
//! ```ignore
//! use nightshade::ecs::camera::{Camera, Projection, PerspectiveCamera};
//! use nightshade::ecs::world::CAMERA;
//!
//! let entity = spawn_entities(world, CAMERA, 1)[0];
//! world.set(entity, Camera {
//!     projection: Projection::Perspective(PerspectiveCamera {
//!         y_fov_rad: 60.0_f32.to_radians(),
//!         aspect_ratio: Some(16.0 / 9.0),
//!         z_near: 0.1,
//!         z_far: Some(1000.0),
//!     }),
//!     smoothing: None,
//! });
//! world.set(entity, LocalTransform {
//!     translation: Vec3::new(0.0, 5.0, 10.0),
//!     ..Default::default()
//! });
//! world.expect_resource::<crate::ecs::camera::resources::ActiveCamera>().0 = Some(entity);
//! ```
//!
//! # Pan-Orbit Camera (Editor-Style)
//!
//! The [`PanOrbitCamera`] provides arc-ball navigation around a focus point:
//!
//! ```ignore
//! use nightshade::ecs::camera::{Camera, PanOrbitCamera};
//! use nightshade::ecs::world::PAN_ORBIT_CAMERA;
//!
//! let entity = spawn_entities(world, PAN_ORBIT_CAMERA, 1)[0];
//! world.set(entity, Camera::default());
//! world.set(entity, PanOrbitCamera::new(
//!     Vec3::zeros(),  // Focus point
//!     15.0,           // Distance from focus
//! )
//!     .with_yaw_pitch(0.5, 0.3)  // Initial angles in radians
//!     .with_zoom_limits(1.0, Some(100.0))  // Min/max zoom
//!     .with_smoothness(0.1, 0.02, 0.1)  // Orbit, pan, zoom smoothing
//! );
//!
//! world.expect_resource::<crate::ecs::camera::resources::ActiveCamera>().0 = Some(entity);
//! ```
//!
//! # Mouse and Gamepad Controls
//!
//! `PanOrbitCamera` responds to input automatically when using the camera system:
//!
//! | Input | Action |
//! |-------|--------|
//! | Left mouse drag | Orbit around focus |
//! | Right mouse drag | Pan focus point |
//! | Scroll wheel | Zoom in/out |
//! | Gamepad right stick | Orbit |
//! | Gamepad left stick | Pan |
//! | Gamepad triggers | Zoom |
//!
//! # Sensitivity Settings
//!
//! ```ignore
//! let mut pan_orbit = PanOrbitCamera::default();
//! pan_orbit.sensitivity.orbit = 0.5;     // Mouse orbit speed
//! pan_orbit.sensitivity.pan = 1.0;       // Mouse pan speed
//! pan_orbit.sensitivity.zoom = 0.8;      // Scroll zoom speed
//! pan_orbit.limits.gamepad_deadzone = 0.15;  // Stick deadzone
//! pan_orbit.smoothness.orbit = 0.1;      // Orbit interpolation (0=instant)
//! pan_orbit.smoothness.pan = 0.02;       // Pan interpolation (0=instant)
//! pan_orbit.smoothness.zoom = 0.1;       // Zoom interpolation (0=instant)
//! ```
//!
//! # Customizing Button Bindings
//!
//! Change which mouse buttons control orbit and pan:
//!
//! ```ignore
//! use nightshade::ecs::camera::{PanOrbitCamera, PanOrbitButton, PanOrbitModifier};
//!
//! // Use middle mouse for orbit, right mouse for pan (like some 3D software)
//! let pan_orbit = PanOrbitCamera::default()
//!     .with_buttons(PanOrbitButton::Middle, PanOrbitButton::Right);
//!
//! // Require Shift key for orbit, Ctrl key for pan
//! let pan_orbit = PanOrbitCamera::default()
//!     .with_modifiers(Some(PanOrbitModifier::Shift), Some(PanOrbitModifier::Control));
//!
//! // Allow camera to go upside down (pitch beyond ±90°)
//! let pan_orbit = PanOrbitCamera::default()
//!     .with_upside_down(true);
//! ```
//!
//! # Touch Controls
//!
//! On touch devices, `PanOrbitCamera` supports:
//!
//! | Gesture | Action |
//! |---------|--------|
//! | Single finger drag | Orbit around focus |
//! | Two finger drag | Pan focus point |
//! | Pinch | Zoom in/out |
//!
//! # Orthographic Camera
//!
//! For 2D games, isometric views, or CAD-style rendering:
//!
//! ```ignore
//! use nightshade::ecs::camera::{Camera, Projection, OrthographicCamera};
//!
//! world.set(entity, Camera {
//!     projection: Projection::Orthographic(OrthographicCamera {
//!         x_mag: 10.0,
//!         y_mag: 10.0,
//!         z_near: 0.1,
//!         z_far: 100.0,
//!     }),
//!     smoothing: None,
//! });
//! ```
//!
//! # Updating FOV at Runtime
//!
//! ```ignore
//! if let Some(camera) = world.get_mut::<crate::ecs::camera::components::Camera>(camera_entity) {
//!     if let Projection::Perspective(ref mut perspective) = camera.projection {
//!         perspective.y_fov_rad = 90.0_f32.to_radians();
//!     }
//! }
//! ```
//!
//! # Setting Focus Point
//!
//! Move the pan-orbit camera to look at a new target:
//!
//! ```ignore
//! if let Some(pan_orbit) = world.get_mut::<crate::ecs::camera::components::PanOrbitCamera>(camera_entity) {
//!     pan_orbit.target_focus = Vec3::new(10.0, 0.0, 5.0);
//!     // Camera will smoothly interpolate to new focus
//! }
//! ```
//!
//! [`Camera`]: components::Camera
//! [`Projection`]: components::Projection
//! [`PanOrbitCamera`]: components::PanOrbitCamera
//! [`Smoothing`]: components::Smoothing

pub mod commands;
pub mod components;
pub mod queries;
pub mod resources;
pub mod systems;

pub use commands::*;
pub use components::*;
pub use queries::*;
pub use systems::*;
