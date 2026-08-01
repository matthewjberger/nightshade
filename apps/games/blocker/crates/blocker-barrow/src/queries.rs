use crate::theme;
use blocker_blocks::queries::footing;
use blocker_blocks::resources::VoxelWorld;
use blocker_world::data::world::places::CAPITALS;
use blocker_world::queries::barrow_floor;
use blocker_world::resources::{Bench, Seat};
use nalgebra_glm::{Vec2, Vec3};

pub fn seat_center(at: [i32; 2]) -> Vec2 {
    Vec2::new(at[0] as f32 + 0.5, at[1] as f32 + 0.5)
}

pub fn seats_awake(
    bench: &Bench,
    eye: Vec3,
    pick: fn(&Seat) -> [i32; 2],
) -> impl Iterator<Item = (usize, [i32; 2])> + '_ {
    bench
        .seats
        .iter()
        .enumerate()
        .filter_map(move |(index, seat)| {
            let at = pick(seat);
            let center = seat_center(at);
            let away = Vec2::new(eye.x - center.x, eye.z - center.y).magnitude();
            (away <= theme::BARROW_WAKE).then_some((index, at))
        })
}

pub fn cell_place(bench: &Bench, at: [i32; 2], span: i32, cell: [i32; 2]) -> Vec3 {
    let half = span / 2;
    let floor = barrow_floor(bench, at);
    Vec3::new(
        (at[0] - half + cell[0]) as f32 + 0.5,
        floor as f32,
        (at[1] - half + cell[1]) as f32 + 0.5,
    )
}

pub fn stand_in_cell(
    world: &VoxelWorld,
    bench: &Bench,
    at: [i32; 2],
    span: i32,
    cell: [i32; 2],
) -> Option<Vec3> {
    let place = cell_place(bench, at, span, cell);
    let floor = footing(
        world,
        place.x.floor() as i32,
        place.z.floor() as i32,
        place.y.floor() as i32,
    )?;
    Some(Vec3::new(place.x, floor as f32 + 1.0, place.z))
}

pub fn nearest_seat(bench: &Bench, place: Vec3) -> usize {
    bench
        .seats
        .iter()
        .enumerate()
        .map(|(index, seat)| {
            let center = seat_center(seat.city);
            let offset = Vec2::new(place.x - center.x, place.z - center.y);
            (offset.magnitude(), index)
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, index)| index)
        .unwrap_or(0)
}

pub fn nearest_barrow(bench: &Bench, place: Vec3, reach: f32) -> Option<usize> {
    bench
        .seats
        .iter()
        .enumerate()
        .map(|(index, seat)| {
            let center = seat_center(seat.barrow);
            let offset = Vec2::new(place.x - center.x, place.z - center.y);
            (offset.magnitude(), index)
        })
        .filter(|(distance, _)| *distance < reach)
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, index)| index)
}

pub fn capital_name(index: usize) -> &'static str {
    CAPITALS
        .get(index)
        .map(|capital| capital.name)
        .unwrap_or("the city")
}
