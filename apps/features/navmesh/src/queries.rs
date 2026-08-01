use crate::commands::{NoiseConfig, NoiseType, TerrainConfig, TerrainGenerationResult};
use nalgebra_glm::{Vec3, vec3};
use nightshade::prelude::*;
use nightshade_render::components::mesh::{Mesh, Vertex};
use noise::{Billow, Fbm, MultiFractal, NoiseFn, Perlin, RidgedMulti, Simplex};

pub fn generate_terrain_mesh(config: &TerrainConfig) -> TerrainGenerationResult {
    let vertex_count = (config.resolution_x * config.resolution_z) as usize;
    let mut vertices = Vec::with_capacity(vertex_count);
    let mut heights = Vec::with_capacity(vertex_count);

    let step_x = config.width / (config.resolution_x - 1) as f32;
    let step_z = config.depth / (config.resolution_z - 1) as f32;
    let half_width = config.width / 2.0;
    let half_depth = config.depth / 2.0;

    let mut min_height = f32::MAX;
    let mut max_height = f32::MIN;

    for z in 0..config.resolution_z {
        for x in 0..config.resolution_x {
            let world_x = x as f32 * step_x - half_width;
            let world_z = z as f32 * step_z - half_depth;

            let height = sample_noise(world_x as f64, world_z as f64, &config.noise) as f32
                * config.height_scale;

            min_height = min_height.min(height);
            max_height = max_height.max(height);

            heights.push(height);

            let u = (x as f32 / (config.resolution_x - 1) as f32) * config.uv_scale[0];
            let v = (z as f32 / (config.resolution_z - 1) as f32) * config.uv_scale[1];

            vertices.push(Vertex::with_tex_coords(
                vec3(world_x, height, world_z),
                vec3(0.0, 1.0, 0.0),
                [u, v],
            ));
        }
    }

    let mut indices =
        Vec::with_capacity(((config.resolution_x - 1) * (config.resolution_z - 1) * 6) as usize);

    for z in 0..(config.resolution_z - 1) {
        for x in 0..(config.resolution_x - 1) {
            let top_left = z * config.resolution_x + x;
            let top_right = top_left + 1;
            let bottom_left = (z + 1) * config.resolution_x + x;
            let bottom_right = bottom_left + 1;

            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);

            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }

    calculate_terrain_normals(
        &mut vertices,
        &indices,
        config.resolution_x,
        config.resolution_z,
    );

    TerrainGenerationResult {
        mesh: Mesh::new(vertices, indices),
        heights,
        min_height,
        max_height,
    }
}

pub fn sample_terrain_height(x: f32, z: f32, config: &TerrainConfig) -> f32 {
    sample_noise(x as f64, z as f64, &config.noise) as f32 * config.height_scale
}

fn sample_noise(x: f64, z: f64, config: &NoiseConfig) -> f64 {
    let scaled_x = x * config.frequency;
    let scaled_z = z * config.frequency;

    match config.noise_type {
        NoiseType::Perlin => {
            let noise = Fbm::<Perlin>::new(config.seed)
                .set_octaves(config.octaves)
                .set_lacunarity(config.lacunarity)
                .set_persistence(config.persistence);
            noise.get([scaled_x, scaled_z])
        }
        NoiseType::Simplex => {
            let noise = Fbm::<Simplex>::new(config.seed)
                .set_octaves(config.octaves)
                .set_lacunarity(config.lacunarity)
                .set_persistence(config.persistence);
            noise.get([scaled_x, scaled_z])
        }
        NoiseType::Billow => {
            let noise = Billow::<Perlin>::new(config.seed)
                .set_octaves(config.octaves)
                .set_lacunarity(config.lacunarity)
                .set_persistence(config.persistence);
            noise.get([scaled_x, scaled_z])
        }
        NoiseType::RidgedMulti => {
            let noise = RidgedMulti::<Perlin>::new(config.seed)
                .set_octaves(config.octaves)
                .set_lacunarity(config.lacunarity);
            noise.get([scaled_x, scaled_z])
        }
    }
}

fn calculate_terrain_normals(
    vertices: &mut [Vertex],
    indices: &[u32],
    resolution_x: u32,
    resolution_z: u32,
) {
    let vertex_count = (resolution_x * resolution_z) as usize;
    let mut normals: Vec<Vec3> = vec![Vec3::zeros(); vertex_count];

    for triangle in indices.chunks(3) {
        let i0 = triangle[0] as usize;
        let i1 = triangle[1] as usize;
        let i2 = triangle[2] as usize;

        let v0 = vec3(
            vertices[i0].position[0],
            vertices[i0].position[1],
            vertices[i0].position[2],
        );
        let v1 = vec3(
            vertices[i1].position[0],
            vertices[i1].position[1],
            vertices[i1].position[2],
        );
        let v2 = vec3(
            vertices[i2].position[0],
            vertices[i2].position[1],
            vertices[i2].position[2],
        );

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let face_normal = nalgebra_glm::cross(&edge1, &edge2);

        normals[i0] += face_normal;
        normals[i1] += face_normal;
        normals[i2] += face_normal;
    }

    for (index, vertex) in vertices.iter_mut().enumerate() {
        let normal = nalgebra_glm::normalize(&normals[index]);
        vertex.normal = [normal.x, normal.y, normal.z];
    }
}

pub fn name_to_color(name: &str) -> nalgebra_glm::Vec4 {
    let mut hash: u32 = 0;
    for byte in name.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }

    let hue = (hash % 360) as f32;
    let saturation = 0.7;
    let value = 0.9;

    let chroma = value * saturation;
    let second = chroma * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let match_value = value - chroma;

    let (red, green, blue) = match (hue / 60.0) as u32 {
        0 => (chroma, second, 0.0),
        1 => (second, chroma, 0.0),
        2 => (0.0, chroma, second),
        3 => (0.0, second, chroma),
        4 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };

    nalgebra_glm::vec4(
        red + match_value,
        green + match_value,
        blue + match_value,
        1.0,
    )
}
