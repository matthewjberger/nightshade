use crate::data;
use crate::queries::cylinder;
use crate::queries::town::{cell_of, center, ground};
use crate::resources::Town;
use nalgebra_glm::Vec2;

pub fn water_near(town: &Town, feet: Vec2) -> Option<Vec2> {
    let (column, row) = cell_of(town, feet)?;
    let reach = data::CAST_RANGE.ceil() as i32;
    let mut best: Option<(f32, Vec2)> = None;
    for step_row in -reach..=reach {
        for step_column in -reach..=reach {
            let near_column = column as i32 + step_column;
            if near_column < 0 || near_column >= town.columns as i32 {
                continue;
            }
            let near_row = (row as i32 + step_row).rem_euclid(town.rows as i32) as u32;
            if ground(town, near_column as u32, near_row) != data::Ground::Water {
                continue;
            }
            let spot = center(near_column as u32, near_row);
            let distance = cylinder::town_distance(feet, spot);
            if distance <= data::CAST_RANGE && best.is_none_or(|(near, _)| distance < near) {
                best = Some((distance, spot));
            }
        }
    }
    best.map(|(_, spot)| spot)
}

pub fn shop_doors() -> Vec<(usize, Vec2)> {
    let mut doors = Vec::new();
    for (index, cottage) in data::COTTAGES.iter().enumerate() {
        if !cottage.awning {
            continue;
        }
        let face = if cottage.door_west {
            cottage.column
        } else {
            cottage.column + data::COTTAGE_WIDTH - 1
        };
        let step = if cottage.door_west { -1.0 } else { 1.0 };
        let row = cottage.row + data::COTTAGE_DEPTH / 2;
        doors.push((index, center(face, row) + Vec2::new(step, 0.0)));
    }
    doors
}

pub fn shop_name(index: usize) -> &'static str {
    match index {
        5 => "the general store",
        _ => "the market stall",
    }
}
