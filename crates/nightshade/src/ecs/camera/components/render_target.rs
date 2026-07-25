/// Where a camera's image goes, and how big it is.
///
/// Without this, "render this camera at this size" has no single expression, and
/// the answer gets borrowed from whichever nearby mechanism happens to carry the
/// right shape: a list on the retained UI to say a camera should be drawn at
/// all, an editor tile rectangle to say how large, a flag on the window to say
/// the window is not the destination. Those are three different concepts
/// standing in for one, and none of them means what it is being used for.
///
/// A camera carrying this is dispatched every frame at [`size`](Self::size),
/// whatever the window is doing.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, enum2schema::Schema,
)]
pub struct RenderTarget {
    /// Render resolution in pixels, independent of the window.
    pub width: u32,
    /// Render resolution in pixels, independent of the window.
    pub height: u32,
    /// Who owns the texture behind it.
    pub ownership: RenderTargetOwnership,
}

/// Who allocates the texture a camera renders into.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, enum2schema::Schema,
)]
pub enum RenderTargetOwnership {
    /// The renderer allocates and resizes it, and the host samples the result.
    /// This is an ordinary offscreen viewport.
    Renderer,
    /// The host allocates it and binds it before each frame through
    /// `WgpuRenderer::set_external_camera_viewport`, because the texture comes
    /// from somewhere the renderer cannot allocate: one array layer of an
    /// OpenXR swapchain image, for instance.
    ///
    /// A frame with one of these is going somewhere other than the window, so
    /// it still renders when the window is minimized or gone.
    Host,
}

impl Default for RenderTarget {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            ownership: RenderTargetOwnership::Renderer,
        }
    }
}

impl RenderTarget {
    /// A target the renderer allocates, at the given size.
    pub fn renderer_owned(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ownership: RenderTargetOwnership::Renderer,
        }
    }

    /// A target the host binds each frame, at the given size.
    pub fn host_owned(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ownership: RenderTargetOwnership::Host,
        }
    }

    /// Size as the renderer expects it.
    pub fn size(&self) -> (u32, u32) {
        (self.width.max(1), self.height.max(1))
    }

    /// Whether the host binds the texture rather than the renderer allocating it.
    pub fn is_host_owned(&self) -> bool {
        matches!(self.ownership, RenderTargetOwnership::Host)
    }
}
