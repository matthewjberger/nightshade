use nalgebra_glm;
use nalgebra_glm::Vec2;

use nightshade_paint::data::Rect;

use super::data_types::SplitDirection;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct TileId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropZone {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

#[derive(Clone, Debug)]
pub struct DropPreview {
    pub target_tile: TileId,
    pub zone: DropZone,
}

#[derive(Clone, Debug)]
pub enum TileNode {
    Pane {
        content_entity: nightshade::ecs::Entity,
        title: String,
    },
    Split {
        direction: SplitDirection,
        ratio: f32,
        children: [TileId; 2],
    },
    Tabs {
        panes: Vec<TileId>,
        active: usize,
    },
}

#[derive(Clone, Debug, Default)]
pub struct UiTileContainerData {
    pub tiles: Vec<Option<TileNode>>,
    pub root: TileId,
    pub rects: Vec<Rect>,
    pub dragging_splitter: Option<(TileId, f32)>,
    pub pending_tab_drag: Option<(TileId, usize, Vec2)>,
    pub dragging_tab: Option<(TileId, usize, Vec2)>,
    pub drop_preview: Option<DropPreview>,
    pub container_entity: nightshade::ecs::Entity,
    pub splitter_width: f32,
    pub tab_bar_height: f32,
    pub next_free: Vec<usize>,
    pub hovered_close: Option<(TileId, usize)>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum TileLayoutNode {
    Pane {
        title: String,
    },
    Split {
        direction: SplitDirection,
        ratio: f32,
        children: [TileId; 2],
    },
    Tabs {
        panes: Vec<TileId>,
        active: usize,
    },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TileLayout {
    pub nodes: Vec<Option<TileLayoutNode>>,
    pub root: TileId,
}

impl UiTileContainerData {
    pub fn tab_close_width(&self) -> f32 {
        (self.tab_bar_height * 0.75).round()
    }

    pub(crate) fn alloc(&mut self, node: TileNode) -> TileId {
        if let Some(index) = self.next_free.pop() {
            self.tiles[index] = Some(node);
            if index >= self.rects.len() {
                self.rects.resize(self.tiles.len(), Rect::default());
            }
            TileId(index)
        } else {
            let index = self.tiles.len();
            self.tiles.push(Some(node));
            self.rects.push(Rect::default());
            TileId(index)
        }
    }

    pub(crate) fn free(&mut self, tile_id: TileId) {
        self.tiles[tile_id.0] = None;
        self.next_free.push(tile_id.0);
    }

    pub fn get(&self, tile_id: TileId) -> Option<&TileNode> {
        self.tiles.get(tile_id.0).and_then(|tile| tile.as_ref())
    }

    pub fn get_mut(&mut self, tile_id: TileId) -> Option<&mut TileNode> {
        self.tiles.get_mut(tile_id.0).and_then(|tile| tile.as_mut())
    }

    pub(crate) fn find_parent_split(&self, child_id: TileId) -> Option<(TileId, usize)> {
        for (index, tile) in self.tiles.iter().enumerate() {
            if let Some(TileNode::Split { children, .. }) = tile {
                if children[0] == child_id {
                    return Some((TileId(index), 0));
                }
                if children[1] == child_id {
                    return Some((TileId(index), 1));
                }
            }
        }
        None
    }
}
