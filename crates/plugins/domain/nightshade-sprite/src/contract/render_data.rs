use nalgebra_glm::{Mat4, Vec2, Vec3};

use crate::contract::SpriteFilter;

#[derive(Clone, Debug)]
pub struct SpriteInstance {
    pub atlas: u32,
    pub transform: Mat4,
    pub sort_position: Vec3,
    pub size: Vec2,
    pub anchor: Vec2,
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub color: [f32; 4],
    pub layer: i32,
    pub billboard: bool,
    pub lean: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteAtlasSpec {
    pub index: u32,
    pub size: u32,
    pub filter: SpriteFilter,
}

#[derive(Clone, Debug)]
pub struct SpriteAtlasUpload {
    pub atlas: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Default)]
pub struct SpriteFrame {
    pub instances: Vec<SpriteInstance>,
    pub atlases: Vec<SpriteAtlasSpec>,
    pub uploads: Vec<SpriteAtlasUpload>,
}

pub fn sprite_draw_order(instances: &[SpriteInstance], view: &Mat4) -> Vec<usize> {
    let mut order: Vec<(i32, f32, f32, usize)> = instances
        .iter()
        .enumerate()
        .map(|(index, instance)| {
            let position = nalgebra_glm::vec4(
                instance.sort_position.x,
                instance.sort_position.y,
                instance.sort_position.z,
                1.0,
            );
            let in_view = view * position;
            (instance.layer, in_view.z, instance.sort_position.y, index)
        })
        .collect();
    order.sort_by(|first, second| {
        first
            .0
            .cmp(&second.0)
            .then_with(|| first.1.total_cmp(&second.1))
            .then_with(|| second.2.total_cmp(&first.2))
    });
    order.into_iter().map(|(_, _, _, index)| index).collect()
}
