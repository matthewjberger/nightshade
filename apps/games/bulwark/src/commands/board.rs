use crate::commands::art::sprite_of;
use crate::commands::{purse, spawn};
use crate::components::Flash;
use crate::contract::{Cell, Tower};
use crate::data;
use crate::queries;
use crate::resources::{Art, Bulwark};
use crate::tags::{Gate, Ground};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_scene::prelude::Visibility;
use nightshade_sprite::prelude::spawn_sprite;

pub fn lay_the_board(commands: &mut Commands, art: &Art, board: usize) -> Entity {
    for row in 0..data::ROWS {
        for column in 0..data::COLUMNS {
            let cell = Cell::new(column, row);
            let center = queries::cell_center(cell);
            let ground = if queries::is_road(board, cell) {
                art.road[queries::road_mask(board, cell) as usize]
            } else if let Some(scenery) = queries::scenery_on(board, cell) {
                art.scenery[scenery.slot()]
            } else {
                art.grass
            };
            let patch = spawn_sprite(
                commands,
                sprite_of(ground, theme::LAYER_GROUND).with_color(queries::grass_tint(cell)),
                Vec3::new(center.x, center.y, 0.0),
            );
            commands.add_tag_type::<Ground>(patch);
        }
    }
    let head = queries::spawn_center(board);
    let gate = spawn_sprite(
        commands,
        sprite_of(art.marker, theme::LAYER_MARKER).with_color(theme::GATE_COLOR),
        Vec3::new(head.x, head.y, 0.0),
    );
    commands.add_tag_type::<Gate>(gate);
    commands.add_tag_type::<Ground>(gate);

    let keep = queries::cell_center(queries::keep_cell(board));
    let stronghold = spawn_sprite(
        commands,
        sprite_of(art.keep, theme::LAYER_STANDING),
        Vec3::new(keep.x, keep.y, 0.0),
    );
    commands.set(stronghold, Flash(0.0));
    commands.add_tag_type::<Ground>(stronghold);
    stronghold
}

pub fn raise_a_wall(commands: &mut Commands, bulwark: &mut Bulwark, wave: usize, count: usize) {
    let art = bulwark.art;
    let board = bulwark.board;
    let level = queries::wall_level(wave);
    let mut raised = 0;
    for _ in 0..theme::CELL_SIDES {
        for road in queries::board_of(board).road.iter().copied() {
            if raised >= count {
                return;
            }
            let Some(beside) = queries::cells_beside(road)
                .into_iter()
                .find(|cell| queries::buildable(bulwark, *cell))
            else {
                continue;
            };
            let kind = Tower::ALL[raised % Tower::ALL.len()];
            let tower = spawn::raise_a_tower(commands, &art, beside, kind);
            purse::take_the_cell(bulwark, beside, tower, 0);
            if level > 1 {
                spawn::raise_the_level(commands, &art, tower, kind, level);
            }
            raised += 1;
        }
    }
}

pub fn raise_the_backdrop(commands: &mut Commands, art: &Art) {
    spawn_sprite(
        commands,
        sprite_of(art.backdrop, theme::LAYER_BACKDROP)
            .with_size(theme::BACKDROP_WIDTH, theme::BACKDROP_HEIGHT),
        Vec3::new(theme::CAMERA_AT.x, theme::CAMERA_AT.y, 0.0),
    );
    spawn_sprite(
        commands,
        sprite_of(art.shade, theme::LAYER_BACKDROP)
            .with_size(theme::BOARD_SHADOW_WIDTH, theme::BOARD_SHADOW_HEIGHT)
            .with_color(theme::BOARD_SHADOW_COLOR),
        Vec3::new(0.0, theme::BOARD_SHADOW_AT, 0.0),
    );
}

pub fn raise_the_marker(commands: &mut Commands, art: &Art) -> Entity {
    let marker = spawn_sprite(
        commands,
        sprite_of(art.marker, theme::LAYER_MARKER),
        Vec3::new(0.0, 0.0, 0.0),
    );
    commands.set(marker, Visibility { visible: false });
    marker
}

pub fn raise_the_ghost(commands: &mut Commands, art: &Art) -> Entity {
    let ghost = spawn_sprite(
        commands,
        sprite_of(art.towers[0][0], theme::LAYER_STANDING).with_color(theme::GHOST_COLOR),
        Vec3::new(0.0, 0.0, 0.0),
    );
    commands.set(ghost, Visibility { visible: false });
    ghost
}

pub fn raise_the_ring(commands: &mut Commands, art: &Art) -> Entity {
    let ring = spawn_sprite(
        commands,
        sprite_of(art.ring, theme::LAYER_MARKER).with_color(theme::RING_COLOR),
        Vec3::new(0.0, 0.0, 0.0),
    );
    commands.set(ring, Visibility { visible: false });
    ring
}
