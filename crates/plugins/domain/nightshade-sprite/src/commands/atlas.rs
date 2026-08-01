use crate::contract::render_data::SpriteAtlasUpload;
use crate::contract::{Sprite, SpriteFilter, SpriteRegion, Tilemap};
use crate::data::ShelfPacker;
use crate::resources::{SpriteAtlas, SpriteAtlases};
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::Entity;
use nightshade::ecs::system_param::Commands;
use nightshade_scene::contract::primitives::{Name, Visibility};
use nightshade_scene::contract::{GlobalTransform, LocalTransform};
use std::collections::HashMap;

pub fn sprite_atlas_create(
    atlases: &mut SpriteAtlases,
    name: impl Into<String>,
    size: u32,
    filter: SpriteFilter,
) -> u32 {
    let name = name.into();
    if let Some(index) = atlases.by_name.get(&name) {
        return *index;
    }
    let index = atlases.atlases.len() as u32;
    atlases.atlases.push(SpriteAtlas {
        name: name.clone(),
        size,
        filter,
        packer: ShelfPacker::new(size),
        regions: HashMap::new(),
        pending: Vec::new(),
    });
    atlases.by_name.insert(name, index);
    index
}

pub fn sprite_atlas_insert(
    atlases: &mut SpriteAtlases,
    atlas: &str,
    name: impl Into<String>,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Option<SpriteRegion> {
    let index = atlases.index_of(atlas)?;
    let atlas = atlases.atlases.get_mut(index as usize)?;
    if rgba.len() != (width * height * 4) as usize {
        return None;
    }
    let name = name.into();
    if let Some(region) = atlas.regions.get(&name) {
        return Some(*region);
    }
    let (x, y) = atlas.packer.allocate(width, height)?;
    let size = atlas.size as f32;
    let region = SpriteRegion::new(
        Vec2::new(x as f32 / size, y as f32 / size),
        Vec2::new((x + width) as f32 / size, (y + height) as f32 / size),
    );
    atlas.regions.insert(name, region);
    atlas.pending.push(SpriteAtlasUpload {
        atlas: index,
        x,
        y,
        width,
        height,
        rgba: rgba.to_vec(),
    });
    Some(region)
}

pub fn spawn_sprite(commands: &mut Commands, sprite: Sprite, position: Vec3) -> Entity {
    let local_transform = LocalTransform {
        translation: position,
        ..Default::default()
    };
    commands.spawn((
        Name(format!("Sprite {}", sprite.atlas)),
        local_transform,
        GlobalTransform(local_transform.as_matrix()),
        Visibility { visible: true },
        sprite,
    ))
}

pub fn spawn_tilemap(commands: &mut Commands, tilemap: Tilemap, position: Vec3) -> Entity {
    let local_transform = LocalTransform {
        translation: position,
        ..Default::default()
    };
    commands.spawn((
        Name(format!("Tilemap {}", tilemap.atlas)),
        local_transform,
        GlobalTransform(local_transform.as_matrix()),
        Visibility { visible: true },
        tilemap,
    ))
}

pub fn sprite_atlas_insert_picture(
    atlases: &mut SpriteAtlases,
    atlas: &str,
    name: impl Into<String>,
    encoded: &[u8],
) -> Option<SpriteRegion> {
    let decoded = nightshade_render::data::decoded_image::decode_to_rgba8(encoded).ok()?;
    sprite_atlas_insert(
        atlases,
        atlas,
        name,
        &decoded.rgba,
        decoded.width,
        decoded.height,
    )
}
