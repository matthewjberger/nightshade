use crate::data::{TREE_COUNT, TreeObstacle};
use crate::queries;
use crate::resources::{Ground, Scene, Trees};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_navmesh::commands::generation::{RecastNavMeshConfig, generate_navmesh_recast};

pub fn spawn_terrain(scene: &mut Scene<'_, '_>, ground: &Ground) {
    let terrain_material = Material {
        base_color: [0.08, 0.12, 0.05, 1.0],
        roughness: 0.95,
        metallic: 0.0,
        ..Default::default()
    };

    crate::commands::spawn_terrain(scene, ground.0.clone(), Vec3::zeros(), terrain_material);
}

pub fn spawn_trees(scene: &mut Scene<'_, '_>, ground: &Ground, trees: &mut Trees) {
    for index in 0..TREE_COUNT {
        let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
        let theta = index as f32 * golden_angle;
        let radius_from_center = (index as f32 / TREE_COUNT as f32).sqrt() * 30.0 + 8.0;
        let x = theta.cos() * radius_from_center;
        let z = theta.sin() * radius_from_center;

        if (x * x + z * z).sqrt() < 6.0 {
            continue;
        }

        let trunk_height = 0.6 + (index as f32 * 0.02) % 0.3;
        let trunk_radius = 0.1 + (index as f32 * 0.002) % 0.06;
        let tree_scale = 0.7 + (index as f32 * 0.015) % 0.5;

        trees.count += 1;

        let terrain_y = queries::sample_terrain_height(x, z, &ground.0);

        trees.obstacles.push(TreeObstacle {
            position: Vec3::new(x, terrain_y, z),
            radius: trunk_radius + 0.3,
            height: trunk_height + 2.0,
        });

        let trunk_material_name = insert_material(
            &mut *scene.stage.materials,
            format!("TreeTrunk_{index}"),
            Material {
                base_color: [0.35, 0.22, 0.12, 1.0],
                roughness: 0.95,
                metallic: 0.0,
                ..Default::default()
            },
        );

        let trunk = scene.storage.spawn_entities::<(
            nightshade_scene::contract::LocalTransform,
            nightshade_scene::contract::GlobalTransform,
            nightshade_render::components::mesh::RenderMesh,
            nightshade_render::components::material::MaterialRef,
            nightshade_render::components::culling::CastsShadow,
            nightshade_scene::contract::primitives::Visibility,
            nightshade_render::data::bounding_volume::BoundingVolume,
        )>(1)[0];
        scene.storage.set(
            trunk,
            LocalTransform {
                translation: Vec3::new(x, terrain_y + trunk_height / 2.0, z),
                rotation: Quat::identity(),
                scale: Vec3::new(trunk_radius * 2.0, trunk_height, trunk_radius * 2.0),
            },
        );
        scene.storage.set(trunk, RenderMesh::new("Cube"));
        scene
            .storage
            .set(trunk, MaterialRef::new(trunk_material_name));
        scene.storage.set(trunk, CastsShadow);

        let green_variation = (index as f32 * 0.02) % 0.12;
        let tier_radii = [2.0 * tree_scale, 1.5 * tree_scale, 0.9 * tree_scale];
        let tier_heights = [1.4 * tree_scale, 1.2 * tree_scale, 1.0 * tree_scale];
        let tier_y_offsets = [0.0, 0.8 * tree_scale, 1.5 * tree_scale];

        for tier in 0..3 {
            let radius = tier_radii[tier];
            let height = tier_heights[tier];
            let y_position = terrain_y + trunk_height + tier_y_offsets[tier] + height / 2.0;

            let cone_material_name = insert_material(
                &mut *scene.stage.materials,
                format!("TreeCone_{index}_{tier}"),
                Material {
                    base_color: [0.1, 0.38 + green_variation, 0.08, 1.0],
                    roughness: 0.95,
                    metallic: 0.0,
                    ..Default::default()
                },
            );

            let cone = scene.storage.spawn_entities::<(
                nightshade_scene::contract::LocalTransform,
                nightshade_scene::contract::GlobalTransform,
                nightshade_render::components::mesh::RenderMesh,
                nightshade_render::components::material::MaterialRef,
                nightshade_render::components::culling::CastsShadow,
                nightshade_scene::contract::primitives::Visibility,
                nightshade_render::data::bounding_volume::BoundingVolume,
            )>(1)[0];
            scene.storage.set(
                cone,
                LocalTransform {
                    translation: Vec3::new(x, y_position, z),
                    rotation: Quat::identity(),
                    scale: Vec3::new(radius, height, radius),
                },
            );
            scene.storage.set(cone, RenderMesh::new("Cone"));
            scene
                .storage
                .set(cone, MaterialRef::new(cone_material_name));
            scene.storage.set(cone, CastsShadow);
        }
    }
}

fn add_box_obstacle(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<[u32; 3]>,
    position: Vec3,
    radius: f32,
    height: f32,
) {
    let base_index = vertices.len() as u32;

    let min_x = position.x - radius;
    let max_x = position.x + radius;
    let min_y = position.y;
    let max_y = position.y + height;
    let min_z = position.z - radius;
    let max_z = position.z + radius;

    vertices.push([min_x, min_y, min_z]);
    vertices.push([max_x, min_y, min_z]);
    vertices.push([max_x, min_y, max_z]);
    vertices.push([min_x, min_y, max_z]);
    vertices.push([min_x, max_y, min_z]);
    vertices.push([max_x, max_y, min_z]);
    vertices.push([max_x, max_y, max_z]);
    vertices.push([min_x, max_y, max_z]);

    indices.push([base_index, base_index + 2, base_index + 1]);
    indices.push([base_index, base_index + 3, base_index + 2]);

    indices.push([base_index + 4, base_index + 5, base_index + 6]);
    indices.push([base_index + 4, base_index + 6, base_index + 7]);

    indices.push([base_index, base_index + 1, base_index + 5]);
    indices.push([base_index, base_index + 5, base_index + 4]);

    indices.push([base_index + 2, base_index + 3, base_index + 7]);
    indices.push([base_index + 2, base_index + 7, base_index + 6]);

    indices.push([base_index, base_index + 4, base_index + 7]);
    indices.push([base_index, base_index + 7, base_index + 3]);

    indices.push([base_index + 1, base_index + 2, base_index + 6]);
    indices.push([base_index + 1, base_index + 6, base_index + 5]);
}

pub fn build_navmesh(scene: &mut Scene<'_, '_>, ground: &Ground, trees: &Trees) {
    let terrain_result = queries::generate_terrain_mesh(&ground.0);

    let mut vertices: Vec<[f32; 3]> = terrain_result
        .mesh
        .vertices
        .iter()
        .map(|vertex| vertex.position)
        .collect();

    let mut indices: Vec<[u32; 3]> = terrain_result
        .mesh
        .indices
        .chunks(3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect();

    for obstacle in &trees.obstacles {
        add_box_obstacle(
            &mut vertices,
            &mut indices,
            obstacle.position,
            obstacle.radius,
            obstacle.height,
        );
    }

    let config = RecastNavMeshConfig {
        agent_radius: 0.5,
        agent_height: 1.8,
        cell_size_fraction: 6.0,
        cell_height_fraction: 12.0,
        walkable_climb: 0.5,
        walkable_slope_angle: std::f32::consts::FRAC_PI_4,
        min_region_size: 4,
        merge_region_size: 10,
        max_simplification_error: 0.5,
        edge_max_len_factor: 6,
        max_vertices_per_polygon: 6,
        detail_sample_dist: 1.0,
        detail_sample_max_error: 0.25,
    };

    match generate_navmesh_recast(&vertices, &indices, &config) {
        Some(navmesh) => {
            tracing::info!(
                "Built navmesh: {} triangles, {} connections",
                navmesh.triangles.len(),
                navmesh
                    .adjacency
                    .values()
                    .map(|list| list.len())
                    .sum::<usize>()
            );
            *scene.stage.navmesh = navmesh;
        }
        None => {
            tracing::warn!("Failed to generate navmesh from terrain");
        }
    }
}
