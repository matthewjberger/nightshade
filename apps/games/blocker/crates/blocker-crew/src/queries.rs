use crate::data::people::Role;
use crate::theme;
use blocker_world::data::world::nodes::WorkKind;
use nalgebra_glm::{Vec2, Vec4};
use nightshade_text::prelude::*;

pub fn mark_properties(color: [f32; 4]) -> TextProperties {
    TextProperties {
        font_size: theme::MARK_SIZE,
        color: Vec4::new(color[0], color[1], color[2], color[3]),
        alignment: TextAlignment::Center,
        vertical_alignment: VerticalAlignment::Middle,
        outline_width: 2.5,
        outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        ..Default::default()
    }
}

pub fn role_at_post(kind: WorkKind) -> Role {
    match kind {
        WorkKind::Market => Role::Trader,
        WorkKind::Well | WorkKind::Hall | WorkKind::Keep | WorkKind::Shrine => Role::Giver,
        WorkKind::Farm => Role::Folk,
    }
}

pub fn spot_in_the_crowd(index: usize, count: usize) -> Vec2 {
    let share = (index as f32 + 0.5) / count.max(1) as f32;
    let turn = index as f32 * theme::CREW_GOLDEN;
    let reach = theme::CREW_SPREAD * share.sqrt();
    Vec2::new(turn.cos(), turn.sin()) * reach
}

pub fn perch_near(
    world: &blocker_blocks::resources::VoxelWorld,
    village: [i32; 2],
    level: i32,
) -> Option<[i32; 3]> {
    let mut best: Option<(i32, [i32; 3])> = None;
    for ring in (6..=theme::PERCH_SEARCH).step_by(4) {
        for probe in 0..12 {
            let turn = probe as f32 / 12.0 * std::f32::consts::TAU;
            let x = village[0] + (turn.cos() * ring as f32).round() as i32;
            let z = village[1] + (turn.sin() * ring as f32).round() as i32;
            let Some(top) = blocker_blocks::queries::walkable(world, x, z) else {
                continue;
            };
            if top - level < theme::PERCH_RISE {
                continue;
            }
            if best.is_none_or(|(height, _)| top > height) {
                best = Some((top, [x, top, z]));
            }
        }
    }
    best.map(|(_, place)| place)
}
