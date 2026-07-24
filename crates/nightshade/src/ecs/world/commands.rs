use crate::ecs::world::{Entity, NAME, World};

use crate::prelude::*;
#[derive(Debug, Clone)]
pub enum EcsCommand {
    DespawnRecursive {
        entity: Entity,
    },
    ReloadMaterial {
        name: String,
        material: Box<crate::render::material::Material>,
    },
}

#[derive(Debug, Clone)]
pub enum RenderCommand {
    UploadUiImageLayer {
        layer: u32,
        rgba_data: Vec<u8>,
        width: u32,
        height: u32,
    },
    LoadHdrSkybox {
        hdr_data: Vec<u8>,
    },
    LoadHdrSkyboxFromPath {
        path: std::path::PathBuf,
    },
    CaptureScreenshot {
        path: Option<std::path::PathBuf>,
        /// When set, the saved PNG is downscaled (preserving aspect) so
        /// the longer side is at most this many pixels. Used for prefab
        /// thumbnails where a full-resolution screenshot is wasteful.
        max_dimension: Option<u32>,
    },
    ReloadTexture {
        name: String,
        rgba_data: Vec<u8>,
        width: u32,
        height: u32,
    },
    /// Upload a 3D color grading lookup table. `data` is RGBA8 for a
    /// 16x16x16 table with red varying fastest then green then blue
    /// (4096 texels, 16384 bytes). Set `color_lut_weight` on the color
    /// grading settings to blend it in.
    SetColorLut {
        data: Vec<u8>,
    },
}

#[derive(Default)]
pub struct CommandQueues {
    pub ecs: Vec<EcsCommand>,
    pub render: Vec<RenderCommand>,
}

/// Spawn `count` entities in the `core` archetype with the supplied component mask.
pub fn spawn_entities(world: &mut World, core_mask: u64, count: usize) -> Vec<Entity> {
    world.ecs.spawn_entities(CORE, core_mask, count)
}

/// Spawn `count` entities in the retained-UI archetype with the supplied component mask.
pub fn spawn_entities_ui(world: &mut World, ui_mask: u64, count: usize) -> Vec<Entity> {
    world.ecs.spawn_entities(UI, ui_mask, count)
}

/// Queue an [`EcsCommand`] to be drained by `process_commands_system` later in the frame schedule.
pub fn queue_ecs_command(world: &mut World, command: EcsCommand) {
    world
        .res_mut::<crate::ecs::world::commands::CommandQueues>()
        .ecs
        .push(command);
}

/// Queue a [`RenderCommand`] to be drained by the renderer at frame setup.
pub fn queue_render_command(world: &mut World, command: RenderCommand) {
    world
        .res_mut::<crate::ecs::world::commands::CommandQueues>()
        .render
        .push(command);
}

/// Lock the OS cursor to the window (relative mouse mode). Used for first-person controls.
pub fn set_cursor_locked(world: &mut World, locked: bool) {
    if let Some(window_handle) = &world.res::<crate::platform::window::Window>().handle {
        if locked {
            if window_handle
                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                .is_err()
            {
                let _ = window_handle.set_cursor_grab(winit::window::CursorGrabMode::Confined);
            }
        } else {
            let _ = window_handle.set_cursor_grab(winit::window::CursorGrabMode::None);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        world
            .res_mut::<crate::platform::window::Window>()
            .cursor_locked = locked;
    }
}

/// Reconcile [`Window::cursor_locked`] with the browser's actual pointer-lock
/// state. On the web the browser, not the app, owns the lock: pressing Escape
/// exits it and the keydown is never delivered, so [`set_cursor_locked`] cannot
/// be the source of truth. Run this once per frame before app systems so a
/// user-initiated release is visible to game logic.
#[cfg(target_arch = "wasm32")]
pub fn sync_cursor_lock_state(world: &mut World) {
    let locked = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.pointer_lock_element())
        .is_some();
    world
        .res_mut::<crate::platform::window::Window>()
        .cursor_locked = locked;
}

/// Show or hide the OS cursor.
pub fn set_cursor_visible(world: &mut World, visible: bool) {
    if let Some(window_handle) = &world.res::<crate::platform::window::Window>().handle {
        window_handle.set_cursor_visible(visible);
    }
}

/// Set the global time scale that the frame's delta time is multiplied by:
/// `0.5` for slow motion, `2.0` for fast forward, `1.0` for real time. Negative
/// values clamp to zero. Systems that read the scaled delta time follow it;
/// unscaled work reads `raw_delta_time` instead.
pub fn set_time_scale(world: &mut World, scale: f32) {
    world.res_mut::<crate::ecs::time::Time>().time_speed = scale.max(0.0);
}

/// The current global time scale.
pub fn time_scale(world: &World) -> f32 {
    world.res::<crate::ecs::time::Time>().time_speed
}

/// Pause game time, so the scaled delta time reports zero until resumed. The
/// time scale is preserved, so resuming returns to the previous speed.
pub fn pause(world: &mut World) {
    world.res_mut::<crate::ecs::time::Time>().paused = true;
}

/// Resume game time after a pause.
pub fn unpause(world: &mut World) {
    world.res_mut::<crate::ecs::time::Time>().paused = false;
}

/// Set whether game time is paused.
pub fn set_paused(world: &mut World, paused: bool) {
    world.res_mut::<crate::ecs::time::Time>().paused = paused;
}

/// Whether game time is currently paused.
pub fn is_paused(world: &World) -> bool {
    world.res::<crate::ecs::time::Time>().paused
}

pub(crate) fn generate_checkerboard_texture() -> (Vec<u8>, u32, u32) {
    let width = 256;
    let height = 256;
    let checker_size = 32;

    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let checker_x = (x / checker_size) % 2;
            let checker_y = (y / checker_size) % 2;
            let is_white = (checker_x + checker_y) % 2 == 0;

            if is_white {
                pixels.extend_from_slice(&[255, 255, 255, 255]);
            } else {
                pixels.extend_from_slice(&[64, 64, 64, 255]);
            }
        }
    }

    (pixels, width, height)
}

pub(crate) fn generate_gradient_texture() -> (Vec<u8>, u32, u32) {
    let width = 256;
    let height = 256;

    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let r = (x * 255 / width) as u8;
            let g = (y * 255 / height) as u8;
            let b = 128u8;
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }

    (pixels, width, height)
}

pub(crate) fn generate_uv_test_texture() -> (Vec<u8>, u32, u32) {
    let width = 256;
    let height = 256;

    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / width as f32;
            let v = y as f32 / height as f32;

            let r = (u * 255.0) as u8;
            let g = (v * 255.0) as u8;
            let b = ((1.0 - u) * (1.0 - v) * 255.0) as u8;

            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }

    (pixels, width, height)
}

pub fn load_hdr_skybox(world: &mut World, hdr_data: Vec<u8>) {
    queue_render_command(world, RenderCommand::LoadHdrSkybox { hdr_data });
}

pub fn load_hdr_skybox_from_path(world: &mut World, path: std::path::PathBuf) {
    queue_render_command(world, RenderCommand::LoadHdrSkyboxFromPath { path });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn capture_screenshot(world: &mut World) {
    queue_render_command(
        world,
        RenderCommand::CaptureScreenshot {
            path: None,
            max_dimension: None,
        },
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn capture_screenshot_to_path(world: &mut World, path: impl Into<std::path::PathBuf>) {
    queue_render_command(
        world,
        RenderCommand::CaptureScreenshot {
            path: Some(path.into()),
            max_dimension: None,
        },
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn capture_thumbnail_to_path(
    world: &mut World,
    path: impl Into<std::path::PathBuf>,
    max_dimension: u32,
) {
    queue_render_command(
        world,
        RenderCommand::CaptureScreenshot {
            path: Some(path.into()),
            max_dimension: Some(max_dimension),
        },
    );
}

pub fn find_entity_by_name(world: &World, name: &str) -> Option<Entity> {
    world.ecs.worlds[CORE].query_entities(NAME).find(|&entity| {
        world
            .get::<crate::ecs::primitives::Name>(entity)
            .is_some_and(|n| n.0 == name)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_commands_system_drains_ecs_and_leaves_render() {
        let mut world = World::default();
        let entity = world.spawn_count(1)[0];

        queue_ecs_command(&mut world, EcsCommand::DespawnRecursive { entity });
        queue_render_command(
            &mut world,
            RenderCommand::LoadHdrSkybox {
                hdr_data: vec![0u8; 4],
            },
        );

        crate::ecs::builtins::process_commands_system(&mut world);

        assert!(
            world
                .res::<crate::ecs::world::commands::CommandQueues>()
                .ecs
                .is_empty(),
            "process_commands_system should drain the ecs queue"
        );
        assert_eq!(
            world
                .res::<crate::ecs::world::commands::CommandQueues>()
                .render
                .len(),
            1,
            "process_commands_system must not touch the render queue"
        );
        assert!(matches!(
            world
                .res::<crate::ecs::world::commands::CommandQueues>()
                .render[0],
            RenderCommand::LoadHdrSkybox { .. }
        ));
    }
}
