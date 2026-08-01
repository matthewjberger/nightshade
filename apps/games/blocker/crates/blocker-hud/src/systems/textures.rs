use crate::resources::Icons;
use crate::resources::{ArtLayers, HotbarPanel};
use blocker_blocks::queries::decode_atlas;
use blocker_blocks::resources::Art;
use nightshade::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::prelude::ui_upload_image;

pub fn repaint_the_icons(
    art: Res<Art>,
    mut icons: ResMut<Icons>,
    mut layers: ResMut<ArtLayers>,
    mut hotbar_panel: ResMut<HotbarPanel>,
    mut build: UiBuild,
) {
    if icons.painted == Some(art.palette) {
        return;
    }
    icons.painted = Some(art.palette);
    let (pixels, width, height) = decode_atlas(art.palette);
    let slot = art.palette.code() as usize;
    if icons.layers[slot].is_none() {
        let mut tree = UiTreeBuilder::new(build.context());
        let (images, runtime) = tree.images_and_runtime();
        icons.layers[slot] =
            ui_upload_image(images, runtime, &pixels, width, height).map(|upload| upload.layer);
    }
    if let Some(layer) = icons.layers[slot] {
        layers.atlas = Some(layer);
        for index in 0..blocker_blocks::data::items::HOTBAR_SLOTS {
            hotbar_panel.shown[index] = None;
        }
    }
}
