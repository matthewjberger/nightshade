use crate::resources::{Boulder, Ground, Noises, Plan};
use crate::theme;
use fast_surface_nets::ndshape::{RuntimeShape, Shape};
use fast_surface_nets::{SurfaceNetsBuffer, surface_nets};
use nightshade::prelude::Vec3;
use nightshade_render::data::geometry::{Mesh, Vertex};
use noise::{NoiseFn, Perlin};

pub fn base_height(noises: &Noises, x: f32, z: f32) -> f32 {
    let at = [x as f64, z as f64];
    let continent = noises.continent.get(at) as f32;
    let hills = noises.hills.get(at) as f32;
    let ridge = 1.0 - (noises.ridge.get(at) as f32).abs();
    let detail = noises.detail.get(at) as f32;
    let rough = noises.rough.get(at) as f32;
    let mountains = ((continent + 0.12) * 2.4).clamp(0.0, 1.0);
    theme::SEA_LEVEL
        + continent * theme::CONTINENT_HEIGHT
        + hills * theme::HILL_HEIGHT
        + ridge * ridge * ridge * theme::RIDGE_HEIGHT * mountains
        + detail * theme::DETAIL_HEIGHT
        + rough * theme::ROUGH_HEIGHT
}

fn carve(noises: &Noises, x: f32, y: f32, z: f32, above: f32) -> f32 {
    let fade = 1.0 - (above / theme::OVERHANG_REACH).abs();
    if fade <= 0.0 {
        return 0.0;
    }
    let wide = [
        x as f64 * theme::WARP_FREQUENCY,
        y as f64 * theme::WARP_FREQUENCY,
        z as f64 * theme::WARP_FREQUENCY,
    ];
    let fine = [
        x as f64 * theme::GRAIN_WARP_FREQUENCY,
        y as f64 * theme::GRAIN_WARP_FREQUENCY,
        z as f64 * theme::GRAIN_WARP_FREQUENCY,
    ];
    let warp = noises.warp.get(wide) as f32;
    let grain = noises.grain.get(fine) as f32;
    let ledge = (y * theme::LEDGE_FREQUENCY).sin() * warp;
    (warp * theme::OVERHANG_DEPTH + grain * theme::FIELD_GRAIN_DEPTH + ledge * theme::LEDGE_DEPTH)
        * fade
}

pub fn tile_of(x: f32, z: f32) -> [i32; 2] {
    [
        (x / theme::TILE_SPAN).floor() as i32,
        (z / theme::TILE_SPAN).floor() as i32,
    ]
}

pub fn tile_origin(key: [i32; 2]) -> Vec3 {
    Vec3::new(
        key[0] as f32 * theme::TILE_SPAN,
        0.0,
        key[1] as f32 * theme::TILE_SPAN,
    )
}

pub fn tile_mesh_name(key: [i32; 2]) -> String {
    format!("terrain {} {}", key[0], key[1])
}

pub fn boulder_mesh_name(kind: usize) -> String {
    format!("terrain boulder {kind}")
}

pub fn reach_of(key: [i32; 2], center: [i32; 2]) -> i32 {
    let across = key[0] - center[0];
    let along = key[1] - center[1];
    across * across + along * along
}

pub fn wanted_tiles(center: [i32; 2], radius: i32) -> Vec<[i32; 2]> {
    let mut keys = Vec::new();
    for along in -radius..=radius {
        for across in -radius..=radius {
            if across * across + along * along > radius * radius {
                continue;
            }
            keys.push([center[0] + across, center[1] + along]);
        }
    }
    keys.sort_by_key(|key| reach_of(*key, center));
    keys
}

pub fn too_far(key: [i32; 2], center: [i32; 2], radius: i32) -> bool {
    let keep = radius + theme::KEEP_MARGIN;
    (key[0] - center[0]).abs() > keep || (key[1] - center[1]).abs() > keep
}

pub fn tile_mesh(noises: &Noises, key: [i32; 2], cells: usize) -> (Mesh, f32, f32) {
    let origin = tile_origin(key);
    let voxel = theme::TILE_SPAN / cells as f32;
    let across = cells as u32 + 2;

    let mut columns = vec![0.0f32; (across * across) as usize];
    let mut lowest = f32::MAX;
    let mut highest = f32::MIN;
    for row in 0..across {
        for column in 0..across {
            let world_x = origin.x + (column as f32 - 1.0) * voxel;
            let world_z = origin.z + (row as f32 - 1.0) * voxel;
            let height = base_height(noises, world_x, world_z);
            columns[(row * across + column) as usize] = height;
            lowest = lowest.min(height);
            highest = highest.max(height);
        }
    }

    let floor = lowest - theme::OVERHANG_REACH - voxel * 2.0;
    let ceiling = highest + theme::OVERHANG_REACH + voxel * 2.0;
    let levels = ((((ceiling - floor) / voxel).ceil() as u32) + 3).min(theme::MAX_LEVELS);

    let shape = RuntimeShape::<u32, 3>::new([across, levels, across]);
    let mut field = vec![0.0f32; shape.size() as usize];
    for row in 0..across {
        for column in 0..across {
            let world_x = origin.x + (column as f32 - 1.0) * voxel;
            let world_z = origin.z + (row as f32 - 1.0) * voxel;
            let ground = columns[(row * across + column) as usize];
            for level in 0..levels {
                let world_y = floor + (level as f32 - 1.0) * voxel;
                let above = world_y - ground;
                let value = above - carve(noises, world_x, world_y, world_z, above);
                field[shape.linearize([column, level, row]) as usize] = value;
            }
        }
    }

    let mut buffer = SurfaceNetsBuffer::default();
    surface_nets(
        &field,
        &shape,
        [0; 3],
        [across - 1, levels - 1, across - 1],
        &mut buffer,
    );
    if buffer.indices.is_empty() {
        return (Mesh::new(Vec::new(), Vec::new()), 0.0, 0.0);
    }

    let mut vertices = Vec::with_capacity(buffer.positions.len());
    let mut under = f32::MAX;
    let mut over = f32::MIN;
    for (place, normal) in buffer.positions.iter().zip(&buffer.normals) {
        let local = Vec3::new(
            (place[0] - 1.0) * voxel,
            floor + (place[1] - 1.0) * voxel,
            (place[2] - 1.0) * voxel,
        );
        under = under.min(local.y);
        over = over.max(local.y);
        let facing = nalgebra_glm::normalize(&Vec3::new(normal[0], normal[1], normal[2]));
        vertices.push(Vertex::with_tex_coords(
            local,
            facing,
            [local.x / theme::TILE_SPAN, local.z / theme::TILE_SPAN],
        ));
    }

    (Mesh::new(vertices, buffer.indices), under, over)
}

pub fn boulder_mesh(seed: u32, segments: u32) -> Mesh {
    let lumps = Perlin::new(seed);
    let grain = Perlin::new(seed.wrapping_add(7));
    let across = segments as usize + 1;
    let mut vertices = Vec::with_capacity(across * across);

    for ring in 0..=segments {
        let down = std::f32::consts::PI * ring as f32 / segments as f32;
        for step in 0..=segments {
            let around = std::f32::consts::TAU * step as f32 / segments as f32;
            let direction = Vec3::new(
                down.sin() * around.cos(),
                down.cos(),
                down.sin() * around.sin(),
            );
            let sample = [
                (direction.x * theme::LUMP_FREQUENCY as f32) as f64,
                (direction.y * theme::LUMP_FREQUENCY as f32) as f64,
                (direction.z * theme::LUMP_FREQUENCY as f32) as f64,
            ];
            let fine = [
                (direction.x * theme::GRAIN_FREQUENCY as f32) as f64,
                (direction.y * theme::GRAIN_FREQUENCY as f32) as f64,
                (direction.z * theme::GRAIN_FREQUENCY as f32) as f64,
            ];
            let reach = 1.0
                + lumps.get(sample) as f32 * theme::LUMP_DEPTH
                + grain.get(fine) as f32 * theme::GRAIN_DEPTH;
            vertices.push(Vertex::with_tex_coords(
                direction * reach,
                direction,
                [step as f32 / segments as f32, ring as f32 / segments as f32],
            ));
        }
    }

    let mut indices = Vec::with_capacity(across * across * 6);
    for ring in 0..segments as usize {
        for step in 0..segments as usize {
            let top_left = (ring * across + step) as u32;
            let top_right = top_left + 1;
            let bottom_left = top_left + across as u32;
            let bottom_right = bottom_left + 1;
            if ring != 0 {
                indices.push(top_left);
                indices.push(bottom_right);
                indices.push(top_right);
            }
            if ring + 1 != segments as usize {
                indices.push(bottom_right);
                indices.push(top_left);
                indices.push(bottom_left);
            }
        }
    }

    let mut mesh = Mesh::new(vertices, indices);
    smooth_the_normals(&mut mesh);
    mesh
}

fn smooth_the_normals(mesh: &mut Mesh) {
    let mut gathered = vec![Vec3::zeros(); mesh.vertices.len()];
    for corners in mesh.indices.as_chunks::<3>().0 {
        let first = Vec3::from_column_slice(&mesh.vertices[corners[0] as usize].position);
        let second = Vec3::from_column_slice(&mesh.vertices[corners[1] as usize].position);
        let third = Vec3::from_column_slice(&mesh.vertices[corners[2] as usize].position);
        let face = (second - first).cross(&(third - first));
        for corner in corners {
            gathered[*corner as usize] += face;
        }
    }
    for (vertex, total) in mesh.vertices.iter_mut().zip(gathered) {
        if total.norm_squared() <= f32::EPSILON {
            continue;
        }
        let smoothed = nalgebra_glm::normalize(&total);
        vertex.normal = [smoothed.x, smoothed.y, smoothed.z];
    }
}

pub fn slope_at(noises: &Noises, x: f32, z: f32, step: f32) -> f32 {
    let left = base_height(noises, x - step, z);
    let right = base_height(noises, x + step, z);
    let back = base_height(noises, x, z - step);
    let front = base_height(noises, x, z + step);
    nalgebra_glm::normalize(&Vec3::new(left - right, 2.0 * step, back - front)).y
}

pub fn scatter_boulders(noises: &Noises, key: [i32; 2], count: usize) -> Vec<Boulder> {
    let mut rng = fastrand::Rng::with_seed(seed_of(key));
    let origin = tile_origin(key);
    let mut standing = Vec::with_capacity(count);
    for _ in 0..count {
        let local_x = rng.f32() * theme::TILE_SPAN;
        let local_z = rng.f32() * theme::TILE_SPAN;
        let world_x = origin.x + local_x;
        let world_z = origin.z + local_z;
        if slope_at(noises, world_x, world_z, 1.5) < theme::BOULDER_SLOPE_LIMIT {
            continue;
        }
        let height = base_height(noises, world_x, world_z);
        let scale = theme::BOULDER_SMALLEST
            + rng.f32() * (theme::BOULDER_LARGEST - theme::BOULDER_SMALLEST);
        standing.push(Boulder {
            place: Vec3::new(local_x, height - scale * theme::BOULDER_SINK, local_z),
            scale,
            yaw: rng.f32() * std::f32::consts::TAU,
            kind: rng.usize(0..theme::BOULDER_KINDS),
        });
    }
    standing
}

fn seed_of(key: [i32; 2]) -> u64 {
    let across = key[0] as i64 as u64;
    let along = key[1] as i64 as u64;
    across
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .rotate_left(31)
        .wrapping_add(along.wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
}

pub fn boulder_triangles(segments: u32) -> usize {
    let segments = segments as usize;
    segments * segments * 2 - segments * 2
}

pub fn field_of(ground: &Ground, _plan: &Plan) -> (usize, usize, usize) {
    let tiles = ground.standing.len();
    let boulders: usize = ground.standing.values().map(|held| held.boulders).sum();
    let terrain: usize = ground.standing.values().map(|held| held.triangles).sum();
    (
        tiles,
        boulders,
        terrain + boulders * boulder_triangles(theme::BOULDER_SEGMENTS),
    )
}
