pub const MAX_LIGHTS: usize = 1024;
pub const MAX_AREA_LIGHTS: usize = 16;

pub const MAX_SPOTLIGHT_SHADOWS: usize = 64;
pub const SPOTLIGHT_ATLAS_SLOTS_PER_ROW: u32 = 8;
pub const SPOTLIGHT_ATLAS_SIZE: u32 = if cfg!(target_arch = "wasm32") {
    1024
} else {
    4096
};

pub const MAX_POINT_LIGHT_SHADOWS: usize = 40;
pub const POINT_SHADOW_FACE_SIZE: u32 = 256;
pub const POINT_SHADOW_NUM_FACES: usize = 6;

pub const CASCADE_ATLAS_SLOTS_PER_ROW: u32 = 2;
pub const CASCADE_ATLAS_SIZE: u32 = if cfg!(target_arch = "wasm32") {
    4096
} else {
    8192
};
pub const CASCADE_SLOT_RESOLUTION: f32 = (CASCADE_ATLAS_SIZE / CASCADE_ATLAS_SLOTS_PER_ROW) as f32;

pub fn point_shadow_layers(device: &wgpu::Device) -> u32 {
    ((POINT_SHADOW_NUM_FACES * MAX_POINT_LIGHT_SHADOWS) as u32)
        .clamp(1, device.limits().max_texture_array_layers)
}

pub fn point_shadow_slots(device: &wgpu::Device) -> usize {
    point_shadow_layers(device) as usize / POINT_SHADOW_NUM_FACES
}

#[derive(Copy, Clone, Debug, Default)]
pub struct BindlessConfig {
    pub enabled: bool,
    pub max_textures: u32,
}
