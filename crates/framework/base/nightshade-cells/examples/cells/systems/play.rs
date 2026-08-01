use crate::resources::Walk;
use nalgebra_glm::{IVec2, vec2};
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_cells::queries::{
    cell_of, center_of, in_bounds, manhattan, neighbours, step_cost, walkable,
};
use nightshade_cells::{CellSize, Cells};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(input: Res<Input>, mut walk: ResMut<Walk>) {
    walk.heading = IVec2::zeros();
    if input.keyboard.just_pressed(KeyCode::ArrowLeft) {
        walk.heading = IVec2::new(-1, 0);
    }
    if input.keyboard.just_pressed(KeyCode::ArrowRight) {
        walk.heading = IVec2::new(1, 0);
    }
    if input.keyboard.just_pressed(KeyCode::ArrowUp) {
        walk.heading = IVec2::new(0, -1);
    }
    if input.keyboard.just_pressed(KeyCode::ArrowDown) {
        walk.heading = IVec2::new(0, 1);
    }
    if input.keyboard.just_pressed(KeyCode::Space) {
        walk.plant = true;
    }
}

pub fn step_the_cells(
    cells: Res<Cells>,
    size: Res<CellSize>,
    mut walk_held: ResMut<Walk>,
    mut readout: ResMut<Readout>,
    storage: &mut Storage,
) {
    let (walker, marker, heading, plant) = (
        walk_held.walker,
        walk_held.marker,
        walk_held.heading,
        walk_held.plant,
    );
    let Some(walker) = walker else {
        return;
    };
    let Some(standing) = storage
        .get::<LocalTransform>(walker)
        .map(|transform| vec2(transform.translation.x, transform.translation.z))
    else {
        return;
    };
    let mut here = cell_of(&size, standing);

    if heading != IVec2::zeros() {
        let wanted = here + heading;
        if in_bounds(&cells, storage, wanted) && walkable(&cells, storage, wanted) {
            let middle = center_of(&size, wanted);
            if let Some(transform) = storage.get_mut::<LocalTransform>(walker) {
                transform.translation.x = middle.x;
                transform.translation.z = middle.y;
            }
            here = wanted;
        }
    }

    if plant {
        walk_held.plant = false;
        walk_held.goal = here;
        if let Some(marker) = marker {
            let middle = center_of(&size, here);
            if let Some(transform) = storage.get_mut::<LocalTransform>(marker) {
                transform.translation.x = middle.x;
                transform.translation.z = middle.y;
            }
        }
    }

    let goal = walk_held.goal;
    let open = neighbours(here)
        .into_iter()
        .filter(|cell| in_bounds(&cells, storage, *cell) && walkable(&cells, storage, *cell))
        .count();
    let cost = step_cost(&cells, storage, here);
    let reach = manhattan(here, goal);

    readout.notes.clear();
    readout
        .notes
        .push(format!("standing in {}, {}", here.x, here.y));
    readout
        .notes
        .push(format!("{open} of four neighbours are walkable"));
    readout
        .notes
        .push(format!("costs {cost:.2}, {reach} cells from the mark"));
}
