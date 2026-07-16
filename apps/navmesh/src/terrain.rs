use nalgebra_glm::{Quat, Vec3, vec3};
use nightshade::ecs::mesh::components::{Mesh, RenderMesh, Vertex};
use nightshade::plugins::physics::components::{
    ColliderComponent, ColliderShape, RigidBodyComponent,
};
use nightshade::plugins::physics::types::InteractionGroups;
use nightshade::prelude::*;
use nightshade::render::bounding_volume::{BoundingVolume, OrientedBoundingBox};
use nightshade::render::material::Material;
use nightshade::render::mesh_cache::mesh_cache_insert;
use noise::{Billow, Fbm, MultiFractal, NoiseFn, Perlin, RidgedMulti, Simplex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum NoiseType {
    #[default]
    Perlin,
    Simplex,
    Billow,
    RidgedMulti,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NoiseConfig {
    pub seed: u32,
    pub frequency: f64,
    pub octaves: usize,
    pub lacunarity: f64,
    pub persistence: f64,
    pub noise_type: NoiseType,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            frequency: 0.02,
            octaves: 4,
            lacunarity: 2.0,
            persistence: 0.5,
            noise_type: NoiseType::Perlin,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainConfig {
    pub width: f32,
    pub depth: f32,
    pub resolution_x: u32,
    pub resolution_z: u32,
    pub height_scale: f32,
    pub noise: NoiseConfig,
    pub uv_scale: [f32; 2],
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            width: 100.0,
            depth: 100.0,
            resolution_x: 64,
            resolution_z: 64,
            height_scale: 10.0,
            noise: NoiseConfig::default(),
            uv_scale: [1.0, 1.0],
        }
    }
}

pub fn sample_terrain_height(x: f32, z: f32, config: &TerrainConfig) -> f32 {
    sample_noise(x as f64, z as f64, &config.noise) as f32 * config.height_scale
}

pub struct TerrainGenerationResult {
    pub mesh: Mesh,
    pub heights: Vec<f32>,
    pub min_height: f32,
    pub max_height: f32,
}

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

pub fn spawn_terrain(
    world: &mut World,
    config: TerrainConfig,
    position: Vec3,
    material: Material,
) -> Entity {
    let TerrainGenerationResult {
        mesh,
        heights,
        min_height,
        max_height,
    } = generate_terrain_mesh(&config);

    let mesh_name = format!("Terrain_{}", rand::random::<u32>());
    mesh_cache_insert(
        &mut world.res_mut::<AssetState>().mesh_cache,
        mesh_name.clone(),
        mesh,
    );

    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | RIGID_BODY
            | COLLIDER
            | VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.get_mut::<nightshade::ecs::primitives::Name>(entity) {
        name.0 = "Terrain".to_string();
    }

    world.set(
        entity,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );

    if let Some(render_mesh) =
        world.get_mut::<nightshade::ecs::mesh::components::RenderMesh>(entity)
    {
        *render_mesh = RenderMesh::new(&mesh_name);
    }

    let heightfield_heights = convert_heights_to_heightfield(
        &heights,
        config.resolution_x as usize,
        config.resolution_z as usize,
    );

    if let Some(rigid_body) =
        world.get_mut::<nightshade::plugins::physics::components::RigidBodyComponent>(entity)
    {
        *rigid_body =
            RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z);
    }

    if let Some(collider) =
        world.get_mut::<nightshade::plugins::physics::components::ColliderComponent>(entity)
    {
        *collider = ColliderComponent {
            handle: None,
            shape: ColliderShape::HeightField {
                nrows: config.resolution_z as usize,
                ncols: config.resolution_x as usize,
                heights: heightfield_heights,
                scale: [config.width, 1.0, config.depth],
            },
            friction: 0.9,
            restitution: 0.0,
            density: 0.0,
            is_sensor: false,
            collision_groups: InteractionGroups::all(),
            solver_groups: InteractionGroups::all(),
        };
    }

    if let Some(bv) = world.get_mut::<nightshade::render::bounding_volume::BoundingVolume>(entity) {
        let half_width = config.width / 2.0;
        let half_depth = config.depth / 2.0;
        let half_height = (max_height - min_height) / 2.0;
        let center_y = (max_height + min_height) / 2.0;

        let sphere_radius =
            (half_width * half_width + half_height * half_height + half_depth * half_depth).sqrt();

        *bv = BoundingVolume {
            obb: OrientedBoundingBox::new(
                Vec3::new(0.0, center_y, 0.0),
                Vec3::new(half_width, half_height, half_depth),
                Quat::identity(),
            ),
            sphere_radius,
        };
    }

    world.set(entity, CastsShadow);
    world.set(entity, Visibility { visible: true });

    set_material(
        world,
        entity,
        format!("TerrainMaterial_{}", entity.id),
        material,
    );

    entity
}

fn convert_heights_to_heightfield(
    heights: &[f32],
    resolution_x: usize,
    resolution_z: usize,
) -> Vec<f32> {
    let mut heightfield_heights = vec![0.0; heights.len()];

    for z in 0..resolution_z {
        for x in 0..resolution_x {
            let mesh_index = z * resolution_x + x;
            let heightfield_index = z + resolution_z * x;
            heightfield_heights[heightfield_index] = heights[mesh_index];
        }
    }

    heightfield_heights
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
