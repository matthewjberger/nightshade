//! Viewport and camera-tile render state. A leaf resource: it names only the
//! renderer's `ViewportRect` config and engine entities, so domains, plugins,
//! and the render driver can all read it without depending on the driver or
//! pulling the renderer into `nightshade-platform`.

/// The active viewport rectangle, the per-camera tile rectangles that drive
/// multi-camera tile rendering, the force-render set, and the tile-render
/// iteration counter. Read by the render driver to build the frame's view
/// inputs and written by apps that compose custom viewport layouts.
pub struct Viewport {
    pub active_viewport_rect: Option<crate::render::config::ViewportRect>,
    pub camera_tile_rects:
        std::collections::HashMap<crate::ecs::world::Entity, crate::render::config::ViewportRect>,
    pub force_render_cameras: std::collections::HashSet<crate::ecs::world::Entity>,
    pub camera_tile_render_iteration: u32,
    /// When false, the runner never creates the renderer: the event loop,
    /// window, and stages run without any GPU work. Set by app compositions
    /// that omit the render plugin. Defaults to true.
    pub create_renderer: bool,
    /// When true, a zero-sized or unavailable window does not stop the frame.
    ///
    /// The window is normally both the output and the thing that says how big
    /// the output is, so minimizing it means there is nothing to draw. That is
    /// wrong for a composition presenting somewhere else, where the window is a
    /// mirror at most: an OpenXR runner still has to render both eyes and meet
    /// the headset's frame deadline while the desktop window is minimized.
    /// Opt-in, so every windowed app keeps idling when minimized. Defaults to
    /// false.
    pub render_without_window: bool,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            active_viewport_rect: None,
            camera_tile_rects: std::collections::HashMap::new(),
            force_render_cameras: std::collections::HashSet::new(),
            camera_tile_render_iteration: 0,
            create_renderer: true,
            render_without_window: false,
        }
    }
}
