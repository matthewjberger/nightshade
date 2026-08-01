pub mod fox;
pub mod interface;
pub mod world;

use crate::queries::generate_terrain_mesh;
use nalgebra_glm::{Quat, Vec3};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_physics::contract::components::{
    ColliderComponent, ColliderShape, RigidBodyComponent,
};
use nightshade_physics::contract::types::InteractionGroups;
use nightshade_render::commands::mesh_cache::mesh_cache_insert;
use nightshade_render::components::mesh::{Mesh, RenderMesh};
use nightshade_render::data::bounding_volume::{BoundingVolume, OrientedBoundingBox};
use nightshade_render::data::material::Material;
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

pub struct TerrainGenerationResult {
    pub mesh: Mesh,
    pub heights: Vec<f32>,
    pub min_height: f32,
    pub max_height: f32,
}

pub fn spawn_terrain(
    scene: &mut crate::resources::Scene<'_, '_>,
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
    mesh_cache_insert(scene.stage.meshes, mesh_name.clone(), mesh);

    let entity = scene.storage.spawn_entities::<(
        nightshade_scene::contract::primitives::Name,
        nightshade_scene::contract::LocalTransform,
        nightshade_scene::contract::GlobalTransform,
        nightshade_render::components::mesh::RenderMesh,
        nightshade_render::components::material::MaterialRef,
        nightshade_render::data::bounding_volume::BoundingVolume,
        nightshade_render::components::culling::CastsShadow,
        nightshade_scene::contract::primitives::Visibility,
    )>(1)[0];

    if let Some(name) = scene
        .storage
        .get_mut::<nightshade_scene::contract::primitives::Name>(entity)
    {
        name.0 = "Terrain".to_string();
    }

    scene.storage.set(
        entity,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );

    if let Some(render_mesh) = scene
        .storage
        .get_mut::<nightshade_render::components::mesh::RenderMesh>(entity)
    {
        *render_mesh = RenderMesh::new(&mesh_name);
    }

    let heightfield_heights = convert_heights_to_heightfield(
        &heights,
        config.resolution_x as usize,
        config.resolution_z as usize,
    );

    scene.storage.set(
        entity,
        RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z),
    );

    scene.storage.set(
        entity,
        ColliderComponent {
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
        },
    );

    if let Some(bv) = scene
        .storage
        .get_mut::<nightshade_render::data::bounding_volume::BoundingVolume>(entity)
    {
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

    scene.storage.set(entity, CastsShadow);
    scene.storage.set(entity, Visibility { visible: true });

    set_material(
        scene.stage.materials,
        scene.commands,
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
