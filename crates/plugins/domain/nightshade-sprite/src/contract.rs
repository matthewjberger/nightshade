pub mod render_data;

use nalgebra_glm::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SpriteFilter {
    #[default]
    Nearest,
    Linear,
}

impl enum2schema::Schema for SpriteFilter {
    fn schema() -> enum2schema::serde_json::Value {
        enum2schema::serde_json::json!({ "type": "string", "enum": ["Nearest", "Linear"] })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
pub struct SpriteRegion {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
}

impl Default for SpriteRegion {
    fn default() -> Self {
        Self::FULL
    }
}

impl SpriteRegion {
    pub const FULL: Self = Self {
        uv_min: Vec2::new(0.0, 0.0),
        uv_max: Vec2::new(1.0, 1.0),
    };

    pub fn new(uv_min: Vec2, uv_max: Vec2) -> Self {
        Self { uv_min, uv_max }
    }

    pub fn cell(&self, columns: u32, rows: u32, index: u32) -> Self {
        let columns = columns.max(1);
        let rows = rows.max(1);
        let column = index % columns;
        let row = (index / columns).min(rows - 1);
        let extent = self.uv_max - self.uv_min;
        let cell = Vec2::new(extent.x / columns as f32, extent.y / rows as f32);
        let uv_min = self.uv_min + Vec2::new(cell.x * column as f32, cell.y * row as f32);
        Self {
            uv_min,
            uv_max: uv_min + cell,
        }
    }

    pub fn cells(&self, columns: u32, rows: u32) -> Vec<Self> {
        (0..columns.max(1) * rows.max(1))
            .map(|index| self.cell(columns, rows, index))
            .collect()
    }

    pub fn flipped(&self, flip_x: bool, flip_y: bool) -> Self {
        let mut region = *self;
        if flip_x {
            std::mem::swap(&mut region.uv_min.x, &mut region.uv_max.x);
        }
        if flip_y {
            std::mem::swap(&mut region.uv_min.y, &mut region.uv_max.y);
        }
        region
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
#[serde(default)]
pub struct Sprite {
    pub atlas: String,
    pub region: SpriteRegion,
    pub size: Vec2,
    pub anchor: Vec2,
    pub color: [f32; 4],
    pub flip_x: bool,
    pub flip_y: bool,
    pub layer: i32,
    pub billboard: bool,
    pub lean: f32,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            atlas: String::new(),
            region: SpriteRegion::FULL,
            size: Vec2::new(1.0, 1.0),
            anchor: Vec2::new(0.5, 0.5),
            color: [1.0, 1.0, 1.0, 1.0],
            flip_x: false,
            flip_y: false,
            layer: 0,
            billboard: false,
            lean: 0.0,
        }
    }
}

impl Sprite {
    pub fn new(atlas: impl Into<String>, region: SpriteRegion) -> Self {
        Self {
            atlas: atlas.into(),
            region,
            ..Default::default()
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    pub fn with_anchor(mut self, anchor: Vec2) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    pub fn with_flip(mut self, flip_x: bool, flip_y: bool) -> Self {
        self.flip_x = flip_x;
        self.flip_y = flip_y;
        self
    }

    pub fn with_lean(mut self, lean: f32) -> Self {
        self.lean = lean;
        self
    }

    pub fn as_billboard(mut self) -> Self {
        self.billboard = true;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
pub struct SpriteAnimationFrame {
    pub region: SpriteRegion,
    pub duration: f32,
}

impl Default for SpriteAnimationFrame {
    fn default() -> Self {
        Self {
            region: SpriteRegion::FULL,
            duration: 0.1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SpriteLoop {
    Once,
    #[default]
    Loop,
    PingPong,
}

impl enum2schema::Schema for SpriteLoop {
    fn schema() -> enum2schema::serde_json::Value {
        enum2schema::serde_json::json!({ "type": "string", "enum": ["Once", "Loop", "PingPong"] })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpriteAnimator {
    pub frames: Vec<SpriteAnimationFrame>,
    pub current_frame: usize,
    pub elapsed: f32,
    pub loop_mode: SpriteLoop,
    pub playing: bool,
    pub speed: f32,
    pub forward: bool,
}

impl Default for SpriteAnimator {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            current_frame: 0,
            elapsed: 0.0,
            loop_mode: SpriteLoop::Loop,
            playing: true,
            speed: 1.0,
            forward: true,
        }
    }
}

impl SpriteAnimator {
    pub fn from_regions(regions: impl IntoIterator<Item = SpriteRegion>, duration: f32) -> Self {
        Self {
            frames: regions
                .into_iter()
                .map(|region| SpriteAnimationFrame { region, duration })
                .collect(),
            ..Default::default()
        }
    }

    pub fn from_grid(
        sheet: SpriteRegion,
        columns: u32,
        rows: u32,
        count: u32,
        duration: f32,
    ) -> Self {
        Self::from_regions(
            (0..count).map(|index| sheet.cell(columns, rows, index)),
            duration,
        )
    }

    pub fn with_loop(mut self, loop_mode: SpriteLoop) -> Self {
        self.loop_mode = loop_mode;
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.elapsed = 0.0;
        self.forward = true;
    }

    pub fn current_region(&self) -> Option<SpriteRegion> {
        self.frames
            .get(self.current_frame)
            .map(|frame| frame.region)
    }

    pub fn advance(&mut self, delta_time: f32) -> bool {
        if !self.playing || self.frames.is_empty() {
            return false;
        }
        self.elapsed += delta_time * self.speed;
        let mut turned = false;
        loop {
            let duration = self.frames[self.current_frame].duration.max(f32::EPSILON);
            if self.elapsed < duration {
                break;
            }
            self.elapsed -= duration;
            turned = true;
            let last = self.frames.len() - 1;
            match self.loop_mode {
                SpriteLoop::Once => {
                    if self.current_frame >= last {
                        self.playing = false;
                        self.elapsed = 0.0;
                        break;
                    }
                    self.current_frame += 1;
                }
                SpriteLoop::Loop => {
                    self.current_frame = if self.current_frame >= last {
                        0
                    } else {
                        self.current_frame + 1
                    };
                }
                SpriteLoop::PingPong => {
                    if self.forward {
                        if self.current_frame >= last {
                            self.forward = false;
                            self.current_frame = last.saturating_sub(1);
                        } else {
                            self.current_frame += 1;
                        }
                    } else if self.current_frame == 0 {
                        self.forward = true;
                        self.current_frame = 1.min(last);
                    } else {
                        self.current_frame -= 1;
                    }
                }
            }
        }
        turned
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, enum2schema::Schema,
)]
pub struct TileData {
    pub id: u32,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl TileData {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            flip_x: false,
            flip_y: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tilemap {
    pub atlas: String,
    pub sheet: SpriteRegion,
    pub sheet_columns: u32,
    pub sheet_rows: u32,
    pub tile_size: Vec2,
    pub columns: u32,
    pub rows: u32,
    pub tiles: Vec<Option<TileData>>,
    pub layer: i32,
    pub color: [f32; 4],
}

impl Default for Tilemap {
    fn default() -> Self {
        Self {
            atlas: String::new(),
            sheet: SpriteRegion::FULL,
            sheet_columns: 1,
            sheet_rows: 1,
            tile_size: Vec2::new(1.0, 1.0),
            columns: 0,
            rows: 0,
            tiles: Vec::new(),
            layer: 0,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl Tilemap {
    pub fn new(atlas: impl Into<String>, columns: u32, rows: u32, tile_size: Vec2) -> Self {
        Self {
            atlas: atlas.into(),
            columns,
            rows,
            tile_size,
            tiles: vec![None; (columns * rows) as usize],
            ..Default::default()
        }
    }

    pub fn with_sheet(mut self, sheet: SpriteRegion, columns: u32, rows: u32) -> Self {
        self.sheet = sheet;
        self.sheet_columns = columns.max(1);
        self.sheet_rows = rows.max(1);
        self
    }

    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    pub fn set_tile(&mut self, column: u32, row: u32, tile: Option<TileData>) {
        if column < self.columns && row < self.rows {
            self.tiles[(row * self.columns + column) as usize] = tile;
        }
    }

    pub fn tile(&self, column: u32, row: u32) -> Option<TileData> {
        if column < self.columns && row < self.rows {
            self.tiles[(row * self.columns + column) as usize]
        } else {
            None
        }
    }

    pub fn fill(&mut self, tile: TileData) {
        self.tiles.fill(Some(tile));
    }

    pub fn tile_region(&self, tile: TileData) -> SpriteRegion {
        self.sheet
            .cell(self.sheet_columns, self.sheet_rows, tile.id)
            .flipped(tile.flip_x, tile.flip_y)
    }

    pub fn tile_center(&self, column: u32, row: u32) -> Vec2 {
        Vec2::new(
            (column as f32 + 0.5) * self.tile_size.x,
            (row as f32 + 0.5) * self.tile_size.y,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TileTrim {
    pub above: u8,
    pub below: u8,
}
