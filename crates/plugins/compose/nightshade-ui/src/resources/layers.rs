#[derive(Default, Debug)]
pub struct UiImageLayerAllocator {
    pub next_layer: u32,
    pub free_layers: Vec<u32>,
}

impl UiImageLayerAllocator {
    pub const MAX_LAYERS: u32 = nightshade_paint::resources::TEXTURE_MAX_LAYERS;

    pub fn allocate(&mut self) -> Option<u32> {
        if let Some(layer) = self.free_layers.pop() {
            return Some(layer);
        }
        if self.next_layer >= Self::MAX_LAYERS {
            return None;
        }
        let layer = self.next_layer;
        self.next_layer += 1;
        Some(layer)
    }

    pub fn release(&mut self, layer: u32) {
        if layer >= Self::MAX_LAYERS {
            return;
        }
        if self.free_layers.contains(&layer) {
            return;
        }
        self.free_layers.push(layer);
    }
}
