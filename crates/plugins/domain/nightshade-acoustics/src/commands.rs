use crate::contract::{AIR, AcousticsGrid, MAX_SIDE};
use crate::queries::acoustics_grid_index;
use nalgebra_glm::Vec3;

pub fn acoustics_grid_lay_out(grid: &mut AcousticsGrid, origin: Vec3, cell: f32, side: u32) {
    grid.origin = origin;
    grid.cell = cell.max(0.01);
    grid.side = side.clamp(1, MAX_SIDE);
    grid.cells = vec![AIR; (grid.side as usize).pow(3)];
    grid.revision = grid.revision.wrapping_add(1);
}

pub fn acoustics_grid_clear(grid: &mut AcousticsGrid) {
    grid.cells.fill(AIR);
    grid.revision = grid.revision.wrapping_add(1);
}

pub fn acoustics_grid_fill(grid: &mut AcousticsGrid, low: Vec3, high: Vec3, material: u8) {
    let first = cell_of(grid, low);
    let last = cell_of(grid, high);
    for z in first[2]..=last[2] {
        for y in first[1]..=last[1] {
            for x in first[0]..=last[0] {
                if let Some(index) = acoustics_grid_index(grid, [x, y, z]) {
                    grid.cells[index] = material;
                }
            }
        }
    }
    grid.revision = grid.revision.wrapping_add(1);
}

fn cell_of(grid: &AcousticsGrid, place: Vec3) -> [u32; 3] {
    let local = (place - grid.origin) / grid.cell;
    let limit = (grid.side.saturating_sub(1)) as f32;
    [
        local.x.clamp(0.0, limit) as u32,
        local.y.clamp(0.0, limit) as u32,
        local.z.clamp(0.0, limit) as u32,
    ]
}

pub fn acoustics_material_set(
    grid: &mut crate::contract::AcousticsGrid,
    code: u8,
    material: crate::contract::MaterialAcoustics,
) {
    let slot = code as usize;
    if slot >= crate::contract::MAX_MATERIALS {
        return;
    }
    grid.materials[slot] = material;
    grid.revision = grid.revision.wrapping_add(1);
}

pub fn acoustics_grid_write(
    grid: &crate::contract::AcousticsGrid,
    path: &std::path::Path,
) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
    writeln!(
        file,
        "origin {} {} {} cell {} side {}",
        grid.origin.x, grid.origin.y, grid.origin.z, grid.cell, grid.side
    )?;
    for (code, material) in grid.materials.iter().enumerate() {
        writeln!(
            file,
            "material {code} absorb {} {} crossing {} {} scatter {} flat {} {} solid {}",
            material.absorb_low,
            material.absorb_high,
            material.crossing_low,
            material.crossing_high,
            material.scatter,
            material.flat_low,
            material.flat_high,
            material.solid
        )?;
    }
    let side = grid.side as usize;
    for (index, cell) in grid.cells.iter().enumerate() {
        if *cell != crate::contract::AIR {
            let across = index % side;
            let up = (index / side) % side;
            let along = index / side / side;
            writeln!(file, "cell {across} {up} {along} {cell}")?;
        }
    }
    file.flush()
}
