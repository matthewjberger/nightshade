#[derive(Default, Clone)]
pub struct TileMap {
    pub width: usize,
    pub height: usize,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy)]
pub struct OpenTile(pub u8);

impl Default for OpenTile {
    fn default() -> Self {
        Self(b' ')
    }
}
