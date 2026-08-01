use nightshade::prelude::*;
use nightshade_graphics::commands::loading::load_texture_from_image_bytes;
use nightshade_graphics::prelude::MaterialRegistry;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::data::texture_data::{SamplerSettings, TextureUsage};

use crate::commands::history::push;
use crate::commands::material::{apply_to_view, write_field};
use crate::commands::notice::report;
use crate::data::MaterialField;
use crate::queries::load::decode_preview;
use crate::resources::{Edit, ViewerResources};

pub fn apply_texture_drop(
    mut viewer: ResMut<ViewerResources>,
    mut registry: ResMut<MaterialRegistry>,
    mut textures: TextureLoad<'_>,
) {
    let viewer = &mut *viewer;
    let drop = viewer
        .incoming
        .texture
        .lock()
        .ok()
        .and_then(|mut slot| slot.take());
    let Some(drop) = drop else {
        return;
    };
    let Some(material_name) = viewer.selection.material.clone() else {
        return;
    };

    let texture_name = format!("Dropped {} {}", viewer.history.frame, drop.name);
    let usage = if drop.slot.is_color() {
        TextureUsage::Color
    } else {
        TextureUsage::Linear
    };
    if load_texture_from_image_bytes(
        &mut textures,
        &texture_name,
        &drop.bytes,
        usage,
        SamplerSettings::default(),
    )
    .is_none()
    {
        report(
            &viewer.incoming.notice,
            format!("{} is not a picture the viewer can read.", drop.name),
        );
        return;
    }
    viewer.model.dropped_textures.push(texture_name.clone());

    let before = viewer.selection.material_values.clone();
    let field = MaterialField::Texture(drop.slot, Some(texture_name.clone()));
    let mut after = before.clone();
    apply_to_view(&mut after, field.clone());
    write_field(&mut registry, &material_name, field);
    push(
        &mut viewer.history,
        Edit::Material {
            name: material_name,
            before,
            after: after.clone(),
        },
    );
    viewer.selection.material_values = after;
    viewer.view.detail_dirty = true;

    if let Some(preview) = decode_preview(texture_name, &drop.bytes)
        && let Ok(mut queue) = viewer.model.previews_decoded.lock()
    {
        queue.push(preview);
    }
}
