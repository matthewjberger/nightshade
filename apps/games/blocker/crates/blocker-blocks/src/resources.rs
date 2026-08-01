use crate::data::blocks::Palette;
use crate::theme;
use nightshade_sprite::prelude::SpriteRegion;
pub use nightshade_voxels::prelude::{
    ChunkStage, ColumnKey, Edits, Hit, VoxelSettings, VoxelViewer, VoxelWorld,
};

pub struct Art {
    pub palette: Palette,
    pub painted: Palette,
}

impl Default for Art {
    fn default() -> Self {
        Self {
            palette: Palette::default(),
            painted: Palette::Kenney,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Sheet {
    pub tiles: SpriteRegion,
    pub empty_rows_below: [u8; theme::SHEET_TILES],
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            tiles: SpriteRegion::FULL,
            empty_rows_below: [0; theme::SHEET_TILES],
        }
    }
}
