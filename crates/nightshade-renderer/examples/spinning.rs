//! A field of spinning primitives, each rotating about its own axis. Shows a
//! per-frame `update` writing each entity's `Transform` straight into the scene
//! world.
//!
//! ```sh
//! cargo run --example spinning --features wgpu,hdr
//! ```

use freecs::Entity;
use nalgebra_glm as glm;
use nightshade_renderer::config::Atmosphere;
use nightshade_renderer::material::Material;
use nightshade_renderer::mesh_cache::mesh_cache_insert;
use nightshade_renderer::procedural_meshes::{
    create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_sphere_mesh,
};
use nightshade_renderer::render_layer::RenderLayer;
use nightshade_renderer::render_world::{
    CullingMask, Layer, MaterialId, MeshName, MorphWeights, Transform, Visible,
};

#[path = "common/driver.rs"]
mod driver;

use driver::{Example, Scene, directional_light};

const GRID: i32 = 6;
const MESHES: [&str; 4] = ["cube", "sphere", "cone", "cylinder"];

struct Spinner {
    entity: Entity,
    base: glm::Vec3,
    axis: glm::Vec3,
    speed: f32,
}

struct Spinning;

impl Example for Spinning {
    type State = Vec<Spinner>;

    fn setup(scene: &mut Scene) -> Vec<Spinner> {
        scene.settings.atmosphere = Atmosphere::Sky;
        scene.settings.show_sky = true;
        scene.settings.ambient_light = [0.4, 0.45, 0.55, 1.0];

        scene.world.spawn(directional_light(
            glm::vec3(-0.4, -1.0, -0.5),
            glm::vec3(1.0, 0.97, 0.9),
            2.5,
        ));

        mesh_cache_insert(
            &mut scene.mesh_cache,
            "cube".to_string(),
            create_cube_mesh(),
        );
        mesh_cache_insert(
            &mut scene.mesh_cache,
            "sphere".to_string(),
            create_sphere_mesh(0.4, 24),
        );
        mesh_cache_insert(
            &mut scene.mesh_cache,
            "cone".to_string(),
            create_cone_mesh(0.4, 0.8, 24),
        );
        mesh_cache_insert(
            &mut scene.mesh_cache,
            "cylinder".to_string(),
            create_cylinder_mesh(0.32, 0.8, 24),
        );

        let mut spinners = Vec::new();
        let half = (GRID - 1) as f32 / 2.0;
        for row in 0..GRID {
            for col in 0..GRID {
                let index = (row * GRID + col) as usize;
                let mesh = MESHES[index % MESHES.len()];
                let phase = index as f32 / (GRID * GRID) as f32 * std::f32::consts::TAU;
                let material = scene.state.render_materials.add(Material {
                    base_color: [
                        0.5 + 0.45 * phase.sin(),
                        0.5 + 0.45 * (phase + 2.1).sin(),
                        0.5 + 0.45 * (phase + 4.2).sin(),
                        1.0,
                    ],
                    roughness: 0.4,
                    ..Material::default()
                });
                let base = glm::vec3(
                    (col as f32 - half) * 1.6,
                    (row as f32 - half) * 1.6 + 3.0,
                    0.0,
                );
                let axis = glm::normalize(&glm::vec3(
                    0.3 + (index % 3) as f32,
                    1.0,
                    0.2 + (index % 2) as f32,
                ));
                let entity = scene.world.spawn((
                    Transform(glm::translation(&base)),
                    MeshName(mesh.to_string()),
                    MaterialId(material),
                    Visible(true),
                    MorphWeights([0.0; 8]),
                    CullingMask(u32::MAX),
                    Layer(RenderLayer::WORLD),
                ));
                spinners.push(Spinner {
                    entity,
                    base,
                    axis,
                    speed: 0.5 + phase.abs() * 0.3,
                });
            }
        }

        scene.camera.position = glm::vec3(0.0, 3.0, 12.0);
        scene.camera.pitch = 0.0;
        spinners
    }

    fn update(scene: &mut Scene, spinners: &mut Vec<Spinner>, time: f32) {
        for spinner in spinners.iter() {
            let transform = glm::translation(&spinner.base)
                * glm::rotation(time * spinner.speed, &spinner.axis);
            scene.world.set(spinner.entity, Transform(transform));
        }
    }
}

fn main() {
    driver::run::<Spinning>("nightshade-renderer: spinning primitives");
}
