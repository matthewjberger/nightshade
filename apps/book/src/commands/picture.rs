use std::collections::HashMap;

use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::prelude::*;

use crate::data::book::IMAGES;
use nightshade_render::data::decoded_image::decode_to_rgba8;

#[derive(Clone, Copy)]
pub struct Picture {
    pub layer: u32,
    pub uv_min: nightshade::prelude::Vec2,
    pub uv_max: nightshade::prelude::Vec2,
    pub width: u32,
    pub height: u32,
}

impl Picture {
    pub fn height_at(&self, width: f32) -> f32 {
        if self.width == 0 {
            return width;
        }
        (width * self.height as f32 / self.width as f32).round()
    }
}

#[derive(Default)]
pub struct Pictures {
    held: HashMap<String, Option<Picture>>,
}

impl Pictures {
    pub fn get(&mut self, tree: &mut UiTreeBuilder, name: &str) -> Option<Picture> {
        if let Some(held) = self.held.get(name) {
            return *held;
        }
        let picture = upload(tree, name);
        self.held.insert(name.to_string(), picture);
        picture
    }
}

fn upload(tree: &mut UiTreeBuilder, name: &str) -> Option<Picture> {
    let source = IMAGES.iter().find(|image| image.name == name)?;
    let decoded = decode_to_rgba8(source.bytes).ok()?;
    let (width, height) = (decoded.width, decoded.height);
    let (images, runtime) = tree.images_and_runtime();
    let upload = ui_upload_image(images, runtime, &decoded.rgba, width, height)?;
    Some(Picture {
        layer: upload.layer,
        uv_min: upload.uv_min,
        uv_max: upload.uv_max,
        width,
        height,
    })
}
