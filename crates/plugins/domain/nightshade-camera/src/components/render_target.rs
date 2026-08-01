#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, enum2schema::Schema,
)]
pub struct RenderTarget {
    pub width: u32,
    pub height: u32,
    pub ownership: RenderTargetOwnership,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, enum2schema::Schema,
)]
pub enum RenderTargetOwnership {
    Renderer,
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
    pub fn renderer_owned(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ownership: RenderTargetOwnership::Renderer,
        }
    }

    pub fn host_owned(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ownership: RenderTargetOwnership::Host,
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width.max(1), self.height.max(1))
    }

    pub fn is_host_owned(&self) -> bool {
        matches!(self.ownership, RenderTargetOwnership::Host)
    }
}
