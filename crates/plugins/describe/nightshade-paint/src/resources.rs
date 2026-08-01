pub const TEXTURE_LAYER_SIZE: u32 = 512;
pub const TEXTURE_MAX_LAYERS: u32 = 128;

pub struct ImageLayerUpload {
    pub layer: u32,
    pub rgba_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Default)]
pub struct PaintFrame {
    pub rects: Vec<crate::data::DrawRect>,
    pub rect_entities: Vec<Option<nightshade::ecs::Entity>>,
    pub images: Vec<crate::data::DrawImage>,
    pub text_meshes: Vec<crate::data::DrawText>,
    pub image_uploads: Vec<ImageLayerUpload>,
}

#[derive(Default)]
pub struct PaintDrawList {
    pub rects: Vec<crate::data::DrawRect>,
    pub rect_entities: Vec<Option<nightshade::ecs::Entity>>,
    pub images: Vec<crate::data::DrawImage>,
    pub text_meshes: Vec<crate::data::DrawText>,
    pub render_slots: RenderSlotAllocator,
}

#[derive(Default, Debug, Clone)]
pub struct RenderSlotAllocator {
    pub rect_slots: std::collections::HashMap<nightshade::ecs::Entity, u32>,
    pub image_slots: std::collections::HashMap<nightshade::ecs::Entity, u32>,
    pub text_slots: std::collections::HashMap<nightshade::ecs::Entity, u32>,
    pub free_rect_slots: Vec<u32>,
    pub free_image_slots: Vec<u32>,
    pub free_text_slots: Vec<u32>,
    pub next_rect_slot: u32,
    pub next_image_slot: u32,
    pub next_text_slot: u32,
    pub dirty_rect_slots: std::collections::HashSet<u32>,
    pub dirty_image_slots: std::collections::HashSet<u32>,
    pub dirty_text_slots: std::collections::HashSet<u32>,
}

#[derive(Default)]
pub struct PaintTextCache {
    pub signatures: std::collections::HashMap<nightshade::ecs::Entity, u64>,
}
