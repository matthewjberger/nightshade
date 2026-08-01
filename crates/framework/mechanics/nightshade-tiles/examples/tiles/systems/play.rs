use crate::resources::Field;
use crate::theme;
use nalgebra_glm::IVec2;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_cells::queries::{in_bounds, walkable};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;
use nightshade_tiles::queries::{at, cells_of, flood};
use nightshade_tiles::{OpenTile, TileMap};

pub fn map_of(storage: &Storage) -> Option<(&TileMap, u8)> {
    storage
        .query_ref::<(&TileMap, &OpenTile)>()
        .iter()
        .next()
        .map(|(_entity, (map, open))| (map, open.0))
}

pub fn take_the_orders(
    input_held: Res<Input>,
    cells: Res<nightshade_cells::Cells>,
    mut field: ResMut<Field>,
    storage: &mut Storage,
) {
    let (step, asked, cleared) = {
        let input = &input_held;
        let mut step = IVec2::zeros();
        if input.keyboard.just_pressed(KeyCode::ArrowLeft) {
            step.x -= 1;
        }
        if input.keyboard.just_pressed(KeyCode::ArrowRight) {
            step.x += 1;
        }
        if input.keyboard.just_pressed(KeyCode::ArrowUp) {
            step.y -= 1;
        }
        if input.keyboard.just_pressed(KeyCode::ArrowDown) {
            step.y += 1;
        }
        (
            step,
            input.keyboard.just_pressed(KeyCode::Space),
            input.keyboard.just_pressed(KeyCode::KeyR),
        )
    };

    if step != IVec2::zeros() {
        let wanted = field.at + step;
        if in_bounds(&cells, storage, wanted) && walkable(&cells, storage, wanted) {
            field.at = wanted;
        }
    }

    if asked {
        let from = field.at;
        if let Some((map, open)) = map_of(storage) {
            field.reached = flood(map, from, open);
        }
    }

    if cleared {
        field.reached.clear();
    }
}

pub fn show_the_field(field: Res<Field>, mut readout: ResMut<Readout>, storage: &mut Storage) {
    let (cells, cursor, here, reached) = (
        field.cells.clone(),
        field.cursor,
        field.at,
        field.reached.clone(),
    );
    let Some((map, _open)) = map_of(storage).map(|(map, open)| (map.clone(), open)) else {
        return;
    };

    let mut spot = None;
    for (cell, tile) in &cells {
        let lit = reached.contains(cell);
        if let Some(transform) = storage.get_mut::<LocalTransform>(*tile) {
            if at(&map, *cell) != theme::WALL {
                transform.translation.y = if lit { theme::LIT } else { theme::FLAT };
            }
            if *cell == here {
                spot = Some(transform.translation);
            }
        }
    }

    if let Some(cursor) = cursor
        && let Some(place) = spot
        && let Some(transform) = storage.get_mut::<LocalTransform>(cursor)
    {
        transform.translation.x = place.x;
        transform.translation.z = place.z;
        transform.translation.y = theme::CURSOR_HEIGHT;
    }

    let open = cells_of(&map, theme::OPEN).len();
    let byte = at(&map, here) as char;
    readout.notes.clear();
    readout
        .notes
        .push(format!("at {}, {} on {byte}", here.x, here.y));
    readout.notes.push(format!("{open} open cells"));
    readout
        .notes
        .push(format!("{} cells reached", reached.len()));
}
