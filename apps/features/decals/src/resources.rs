use nightshade::prelude::*;
use nightshade_render::resources::gpu_picking::GpuPickResult;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum DecalType {
    #[default]
    Spray,
    BulletHole,
    BloodSplatter,
    NeonSign,
    Graffiti,
    AwesomeFace,
}

impl DecalType {
    pub fn name(self) -> &'static str {
        match self {
            DecalType::Spray => "Spray Paint",
            DecalType::BulletHole => "Bullet Hole",
            DecalType::BloodSplatter => "Blood Splatter",
            DecalType::NeonSign => "Neon Sign (Emissive)",
            DecalType::Graffiti => "Graffiti (Emissive)",
            DecalType::AwesomeFace => "Awesome Face",
        }
    }

    pub fn texture(self) -> &'static str {
        match self {
            DecalType::AwesomeFace => crate::theme::AWESOMEFACE_TEXTURE,
            _ => crate::theme::CHECKERBOARD_TEXTURE,
        }
    }

    pub fn color(self) -> [f32; 4] {
        match self {
            DecalType::Spray => [1.0, 0.2, 0.2, 0.9],
            DecalType::BulletHole => [0.2, 0.2, 0.2, 1.0],
            DecalType::BloodSplatter => [0.6, 0.0, 0.0, 0.8],
            DecalType::NeonSign => [0.0, 1.0, 1.0, 1.0],
            DecalType::Graffiti => [1.0, 0.0, 1.0, 1.0],
            DecalType::AwesomeFace => [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn is_emissive(self) -> bool {
        matches!(self, DecalType::NeonSign | DecalType::Graffiti)
    }

    pub fn next(self) -> Self {
        match self {
            DecalType::Spray => DecalType::BulletHole,
            DecalType::BulletHole => DecalType::BloodSplatter,
            DecalType::BloodSplatter => DecalType::NeonSign,
            DecalType::NeonSign => DecalType::Graffiti,
            DecalType::Graffiti => DecalType::AwesomeFace,
            DecalType::AwesomeFace => DecalType::Spray,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => DecalType::Spray,
            1 => DecalType::BulletHole,
            2 => DecalType::BloodSplatter,
            3 => DecalType::NeonSign,
            4 => DecalType::Graffiti,
            _ => DecalType::AwesomeFace,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct UiHandles {
    pub panel: Entity,
    pub type_label: Entity,
    pub decal_size: Entity,
    pub emissive_label: Entity,
    pub emissive_strength: Entity,
    pub show_preview: Entity,
    pub use_awesomeface: Entity,
    pub active_label: Entity,
    pub clear_button: Entity,
    pub bloom_enabled: Entity,
    pub bloom_intensity: Entity,
}

pub struct DecalsResources {
    pub current_decal_type: DecalType,
    pub emissive_strength: f32,
    pub decal_size: f32,
    pub show_controls: bool,
    pub show_preview: bool,
    pub use_awesomeface: bool,
    pub preview_lines_entity: Option<Entity>,
    pub last_pick_result: Option<GpuPickResult>,
    pub last_mouse_position: (u32, u32),
    pub clear_requested: bool,
    pub handles: UiHandles,
    pub last_active_count: usize,
    pub last_decal_type: Option<DecalType>,
}

impl Default for DecalsResources {
    fn default() -> Self {
        Self {
            current_decal_type: DecalType::default(),
            emissive_strength: crate::theme::DEFAULT_EMISSIVE_STRENGTH,
            decal_size: crate::theme::DEFAULT_DECAL_SIZE,
            show_controls: true,
            show_preview: true,
            use_awesomeface: false,
            preview_lines_entity: None,
            last_pick_result: None,
            last_mouse_position: (0, 0),
            clear_requested: false,
            handles: UiHandles::default(),
            last_active_count: usize::MAX,
            last_decal_type: None,
        }
    }
}
