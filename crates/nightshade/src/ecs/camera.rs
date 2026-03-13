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
//! let entity = world.spawn_entities(CAMERA, 1)[0];
//! world.core.set_camera(entity, Camera {
//!     projection: Projection::Perspective(PerspectiveCamera {
//!         y_fov_rad: 60.0_f32.to_radians(),
//!         aspect_ratio: 16.0 / 9.0,
//!         z_near: 0.1,
//!         z_far: 1000.0,
//!     }),
//!     smoothing: None,
//! });
//! world.core.set_local_transform(entity, LocalTransform {
//!     translation: Vec3::new(0.0, 5.0, 10.0),
//!     ..Default::default()
//! });
//! world.core.add_local_transform_dirty(entity);
//! world.resources.active_camera = Some(entity);
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
//! let entity = world.spawn_entities(PAN_ORBIT_CAMERA, 1)[0];
//! world.core.set_camera(entity, Camera::default());
//! world.core.set_pan_orbit_camera(entity, PanOrbitCamera::new(
//!     Vec3::zeros(),  // Focus point
//!     15.0,           // Distance from focus
//! )
//!     .with_yaw_pitch(0.5, 0.3)  // Initial angles in radians
//!     .with_zoom_limits(1.0, Some(100.0))  // Min/max zoom
//!     .with_smoothness(0.1, 0.02, 0.1)  // Orbit, pan, zoom smoothing
//! );
//!
//! world.resources.active_camera = Some(entity);
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
//! pan_orbit.orbit_sensitivity = 0.5;   // Mouse orbit speed
//! pan_orbit.pan_sensitivity = 1.0;     // Mouse pan speed
//! pan_orbit.zoom_sensitivity = 0.8;    // Scroll zoom speed
//! pan_orbit.gamepad_deadzone = 0.15;   // Stick deadzone
//! pan_orbit.orbit_smoothness = 0.1;    // Orbit interpolation (0=instant)
//! pan_orbit.pan_smoothness = 0.02;     // Pan interpolation (0=instant)
//! pan_orbit.zoom_smoothness = 0.1;     // Zoom interpolation (0=instant)
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
//! world.core.set_camera(entity, Camera {
//!     projection: Projection::Orthographic(OrthographicCamera {
//!         left: -10.0,
//!         right: 10.0,
//!         bottom: -10.0,
//!         top: 10.0,
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
//! if let Some(camera) = world.core.get_camera_mut(camera_entity) {
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
//! if let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(camera_entity) {
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
pub mod systems;

pub use commands::*;
pub use components::*;
pub use queries::*;
pub use systems::*;
