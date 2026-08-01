use crate::data::{BUILDING, CLUTTER, DOOR_LEAF, GLASS_ALPHA, GRID_CELL, GRID_ORIGIN, GRID_SIDE};
use nightshade_acoustics::commands::{
    acoustics_grid_clear, acoustics_grid_fill, acoustics_grid_lay_out,
};
use nightshade_acoustics::contract::AcousticsGrid;

pub fn lay_out_the_building(grid: &mut AcousticsGrid, shut: bool) {
    if grid.side == GRID_SIDE {
        acoustics_grid_clear(grid);
    } else {
        acoustics_grid_lay_out(grid, GRID_ORIGIN, GRID_CELL, GRID_SIDE);
    }
    for slab in BUILDING.iter().chain(CLUTTER.iter()) {
        acoustics_grid_fill(grid, slab.low, slab.high, slab.material);
    }
    if shut {
        acoustics_grid_fill(grid, DOOR_LEAF.low, DOOR_LEAF.high, DOOR_LEAF.material);
    }
}

pub fn glaze_the_building(
    registry: &mut nightshade_render::resources::material_registry::MaterialRegistry,
    glassy: bool,
) {
    let alpha = if glassy { GLASS_ALPHA } else { 1.0 };
    let mode = if glassy {
        nightshade_render::data::material::AlphaMode::Blend
    } else {
        nightshade_render::data::material::AlphaMode::Opaque
    };
    for index in 1..BUILDING.len() + CLUTTER.len() {
        glaze_one(registry, &format!("slab_{index}"), alpha, mode);
    }
    glaze_one(registry, "door", alpha, mode);
}

fn glaze_one(
    registry: &mut nightshade_render::resources::material_registry::MaterialRegistry,
    name: &str,
    alpha: f32,
    mode: nightshade_render::data::material::AlphaMode,
) {
    nightshade_render::commands::material_registry::material_registry_mutate(
        registry,
        name,
        |material| {
            material.base_color[3] = alpha;
            material.alpha_mode = mode;
        },
    );
}
