use crate::data;
use nalgebra_glm::Vec2;
use nightshade::prelude::Entity;
use nightshade_sprite::prelude::SpriteRegion;

#[derive(Default, Clone, Copy)]
pub struct Sheets {
    pub tiles: SpriteRegion,
    pub characters: SpriteRegion,
}

impl Sheets {
    pub fn tile(&self, index: u32) -> SpriteRegion {
        self.tiles
            .cell(data::TILE_SHEET_COLUMNS, data::TILE_SHEET_ROWS, index)
    }

    pub fn pose(&self, pose: &data::CharacterPose) -> SpriteRegion {
        let extent = self.characters.uv_max - self.characters.uv_min;
        let texel = Vec2::new(
            extent.x / data::CHARACTER_SHEET_WIDTH,
            extent.y / data::CHARACTER_SHEET_HEIGHT,
        );
        let uv_min = self.characters.uv_min + Vec2::new(pose.x * texel.x, pose.y * texel.y);
        let uv_max = uv_min + Vec2::new(pose.width * texel.x, data::CHARACTER_HEIGHT * texel.y);
        SpriteRegion::new(uv_min, uv_max)
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum FloorPhase {
    #[default]
    Fighting,
    Cleared,
    Dead,
}

pub struct Floor {
    pub columns: u32,
    pub rows: u32,
    pub solid: Vec<bool>,
    pub start: Vec2,
    pub spawns: Vec<Vec2>,
    pub phase: FloorPhase,
    pub remaining: u32,
    pub killed: u32,
    pub elapsed: f32,
    pub restart_timer: f32,
    pub shake: f32,
    pub flash: f32,
    pub rng: fastrand::Rng,
}

impl Default for Floor {
    fn default() -> Self {
        Self {
            columns: 0,
            rows: 0,
            solid: Vec::new(),
            start: Vec2::zeros(),
            spawns: Vec::new(),
            phase: FloorPhase::Fighting,
            remaining: 0,
            killed: 0,
            elapsed: 0.0,
            restart_timer: 0.0,
            shake: 0.0,
            flash: 0.0,
            rng: fastrand::Rng::with_seed(0x9e37_79b9),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Overlay {
    pub readout: Entity,
    pub banner: Entity,
    pub flash: Entity,
}

#[derive(Default)]
pub struct NeonStaticResources {
    pub sheets: Sheets,
    pub floor: Floor,
    pub overlay: Overlay,
    pub round: Vec<Entity>,
    pub enemies: Vec<(Entity, Vec2)>,
    pub player: Option<Entity>,
    pub camera: Option<Entity>,
    pub player_position: Vec2,
    pub camera_anchor: Vec2,
    pub camera_position: Vec2,
    pub readout: String,
    pub banner: String,
    pub last_flash: u32,
}
