use crate::contract::render_data::{SpriteFrame, SpriteInstance};
use crate::contract::{Sprite, Tilemap};
use crate::resources::SpriteAtlases;
use nalgebra_glm::Vec3;
use nightshade::ecs::system_param::{Query, ResMut};
use nightshade_scene::contract::GlobalTransform;
use nightshade_scene::contract::primitives::Visibility;

pub fn gather_sprites_system(
    mut frame: ResMut<SpriteFrame>,
    mut atlases: ResMut<SpriteAtlases>,
    sprites: Query<(&Sprite, &GlobalTransform, Option<&Visibility>)>,
    tilemaps: Query<(&Tilemap, &GlobalTransform, Option<&Visibility>)>,
) {
    frame.instances.clear();
    frame.atlases.clear();
    frame.atlases.extend(atlases.specs());
    for atlas in &mut atlases.atlases {
        frame.uploads.append(&mut atlas.pending);
    }

    sprites.for_each(|_entity, (sprite, global_transform, visibility)| {
        if visibility.is_some_and(|visibility| !visibility.visible) {
            return;
        }
        let Some(atlas) = atlases.index_of(&sprite.atlas) else {
            return;
        };
        let region = sprite.region.flipped(sprite.flip_x, sprite.flip_y);
        frame.instances.push(SpriteInstance {
            atlas,
            transform: global_transform.0,
            sort_position: global_transform.translation(),
            size: sprite.size,
            anchor: sprite.anchor,
            uv_min: region.uv_min,
            uv_max: region.uv_max,
            color: sprite.color,
            layer: sprite.layer,
            billboard: sprite.billboard,
            lean: sprite.lean,
        });
    });

    tilemaps.for_each(|_entity, (tilemap, global_transform, visibility)| {
        if visibility.is_some_and(|visibility| !visibility.visible) {
            return;
        }
        let Some(atlas) = atlases.index_of(&tilemap.atlas) else {
            return;
        };
        let sort_position = global_transform.translation();
        for row in 0..tilemap.rows {
            for column in 0..tilemap.columns {
                let Some(tile) = tilemap.tile(column, row) else {
                    continue;
                };
                let region = tilemap.tile_region(tile);
                let center = tilemap.tile_center(column, row);
                let offset = nalgebra_glm::translation(&Vec3::new(center.x, center.y, 0.0));
                frame.instances.push(SpriteInstance {
                    atlas,
                    transform: global_transform.0 * offset,
                    sort_position,
                    size: tilemap.tile_size,
                    anchor: nalgebra_glm::Vec2::new(0.5, 0.5),
                    uv_min: region.uv_min,
                    uv_max: region.uv_max,
                    color: tilemap.color,
                    layer: tilemap.layer,
                    billboard: false,
                    lean: 0.0,
                });
            }
        }
    });
}
