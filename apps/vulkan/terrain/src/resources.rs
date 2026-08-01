use crate::theme;
use nightshade::prelude::Vec3;
use nightshade_cli::clap;
use nightshade_render::data::geometry::Mesh;
use noise::{Fbm, MultiFractal, Perlin};
use std::collections::HashMap;

#[derive(clap::Args, Default, Clone)]
pub struct Arguments {
    #[arg(long, help = "How many tiles the ground reaches out from the camera")]
    pub radius: Option<i32>,
    #[arg(long, help = "How many cells one tile is divided into along each edge")]
    pub cells: Option<usize>,
    #[arg(long, help = "How many boulders stand on each tile")]
    pub boulders: Option<usize>,
    #[arg(long, help = "The number the ground is made from")]
    pub seed: Option<u32>,
}

pub struct Plan {
    pub radius: i32,
    pub cells: usize,
    pub boulders: usize,
    pub seed: u32,
}

impl Default for Plan {
    fn default() -> Self {
        Self {
            radius: theme::VIEW_RADIUS,
            cells: theme::TILE_CELLS,
            boulders: theme::BOULDERS_PER_TILE,
            seed: theme::SEED,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Standing {
    pub boulders: usize,
    pub triangles: usize,
}

#[derive(Default)]
pub struct Ground {
    pub center: Option<[i32; 2]>,
    pub standing: HashMap<[i32; 2], Standing>,
    pub leaving: Vec<[i32; 2]>,
}

pub struct Noises {
    pub continent: Fbm<Perlin>,
    pub hills: Fbm<Perlin>,
    pub ridge: Fbm<Perlin>,
    pub detail: Fbm<Perlin>,
    pub rough: Fbm<Perlin>,
    pub warp: Fbm<Perlin>,
    pub grain: Fbm<Perlin>,
}

impl Noises {
    pub fn new(seed: u32) -> Self {
        Self {
            continent: Fbm::<Perlin>::new(seed)
                .set_octaves(3)
                .set_frequency(theme::CONTINENT_FREQUENCY),
            hills: Fbm::<Perlin>::new(seed.wrapping_add(17))
                .set_octaves(4)
                .set_frequency(theme::HILL_FREQUENCY),
            ridge: Fbm::<Perlin>::new(seed.wrapping_add(31))
                .set_octaves(3)
                .set_frequency(theme::RIDGE_FREQUENCY),
            detail: Fbm::<Perlin>::new(seed.wrapping_add(53))
                .set_octaves(3)
                .set_frequency(theme::DETAIL_FREQUENCY),
            rough: Fbm::<Perlin>::new(seed.wrapping_add(89))
                .set_octaves(2)
                .set_frequency(theme::ROUGH_FREQUENCY),
            warp: Fbm::<Perlin>::new(seed.wrapping_add(101)).set_octaves(3),
            grain: Fbm::<Perlin>::new(seed.wrapping_add(131)).set_octaves(2),
        }
    }
}

impl Default for Noises {
    fn default() -> Self {
        Self::new(theme::SEED)
    }
}

#[derive(Default)]
pub struct Terrain {
    pub noises: Noises,
}

#[derive(Default)]
pub struct Bench {
    pub made: Option<(u32, Noises)>,
}

pub struct TileJob {
    pub key: [i32; 2],
    pub cells: usize,
    pub boulders: usize,
    pub seed: u32,
}

#[derive(Clone, Copy)]
pub struct Boulder {
    pub place: Vec3,
    pub scale: f32,
    pub yaw: f32,
    pub kind: usize,
}

pub struct TileWork {
    pub key: [i32; 2],
    pub mesh: Mesh,
    pub lowest: f32,
    pub highest: f32,
    pub boulders: Vec<Boulder>,
}
