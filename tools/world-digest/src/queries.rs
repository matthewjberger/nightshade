use crate::data;
use crate::resources::Run;
use fnv::FnvHasher;
use nightshade_voxels::prelude::{ColumnKey, VoxelWorld};
use nightshade_voxels::queries::column_levels;
use std::hash::Hasher;

pub fn read_the_run(arguments: &[String]) -> Result<Run, String> {
    let mut words = arguments.iter();
    let game = match words.next() {
        Some(word) => word.clone(),
        None => return Err(usage()),
    };
    if !data::GAMES.contains(&game.as_str()) {
        return Err(format!("{game} is not one of {:?}", data::GAMES));
    }
    let seed = match words.next() {
        Some(word) => word
            .parse::<u32>()
            .map_err(|_| format!("{word} is not a seed"))?,
        None => data::SEED,
    };
    let radius = match words.next() {
        Some(word) => word
            .parse::<i32>()
            .map_err(|_| format!("{word} is not a radius"))?,
        None => data::RADIUS,
    };
    if radius < 0 {
        return Err("the radius must not be negative".to_string());
    }
    Ok(Run { game, seed, radius })
}

pub fn usage() -> String {
    format!(
        "world-digest <{}> [seed] [radius]\n  seed defaults to {}, radius to {}",
        data::GAMES.join("|"),
        data::SEED,
        data::RADIUS
    )
}

pub fn column_count(radius: i32) -> i32 {
    (radius * 2 + 1) * (radius * 2 + 1)
}

pub fn world_digest(world: &VoxelWorld, columns: &[ColumnKey]) -> u64 {
    let mut digest = FnvHasher::default();
    for &column in columns {
        take_the_column(world, column, &mut digest);
    }
    digest.finish()
}

pub fn column_digest(world: &VoxelWorld, column: ColumnKey) -> u64 {
    let mut digest = FnvHasher::default();
    take_the_column(world, column, &mut digest);
    digest.finish()
}

fn take_the_column(world: &VoxelWorld, column: ColumnKey, digest: &mut FnvHasher) {
    digest.write(&column[0].to_le_bytes());
    digest.write(&column[1].to_le_bytes());
    for level in column_levels() {
        match world.chunks.get(&[column[0], level, column[1]]) {
            Some(chunk) => digest.write(chunk.blocks.as_slice()),
            None => digest.write(&[data::NO_CHUNK]),
        }
    }
    match world.surface.get(&column) {
        Some(heights) => {
            for height in heights.iter() {
                digest.write(&height.to_le_bytes());
            }
        }
        None => digest.write(&[data::NO_SURFACE]),
    }
}
