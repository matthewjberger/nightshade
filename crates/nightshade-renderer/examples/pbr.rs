//! Physically based shading: a grid of spheres running from dielectric to metal
//! (bottom to top) and rough to smooth (left to right) under one directional
//! light and a procedural sky.
//!
//! ```sh
//! cargo run --example pbr --features wgpu,hdr
//! ```

use nalgebra_glm as glm;
use nightshade_renderer::config::Atmosphere;
use nightshade_renderer::material::Material;
use nightshade_renderer::mesh_cache::mesh_cache_insert;
use nightshade_renderer::procedural_meshes::create_sphere_mesh;
use nightshade_renderer::render_layer::RenderLayer;
use nightshade_renderer::render_world::{
    CullingMask, Layer, MaterialId, MeshName, MorphWeights, Transform, Visible,
};
use nightshade_renderer::wgpu::render_configs::SingleViewHost;

#[path = "common/driver.rs"]
mod driver;

use driver::{Camera, Example, directional_light};

const GRID: i32 = 6;

struct Pbr;

impl Example for Pbr {
    type State = ();

    fn setup(host: &mut SingleViewHost, camera: &mut Camera) {
        host.settings.atmosphere = Atmosphere::Sky;
        host.settings.show_sky = true;
        host.settings.ambient_light = [0.35, 0.4, 0.5, 0.6];

        host.scene_world.spawn(directional_light(
            glm::vec3(-0.5, -1.0, -0.6),
            glm::vec3(1.0, 0.97, 0.9),
            2.6,
        ));

        mesh_cache_insert(
            &mut host.mesh_cache,
            "sphere".to_string(),
            create_sphere_mesh(0.45, 32),
        );

        let half = (GRID - 1) as f32 / 2.0;
        for row in 0..GRID {
            for col in 0..GRID {
                let metallic = row as f32 / (GRID - 1) as f32;
                let roughness = (col as f32 / (GRID - 1) as f32).clamp(0.05, 1.0);
                let material = host.state.render_materials.add(Material {
                    base_color: [0.9, 0.22, 0.2, 1.0],
                    metallic,
                    roughness,
                    ..Material::default()
                });
                let x = (col as f32 - half) * 1.1;
                let y = (row as f32 - half) * 1.1 + 2.5;
                host.scene_world.spawn((
                    Transform(glm::translation(&glm::vec3(x, y, 0.0))),
                    MeshName("sphere".to_string()),
                    MaterialId(material),
                    Visible(true),
                    MorphWeights([0.0; 8]),
                    CullingMask(u32::MAX),
                    Layer(RenderLayer::WORLD),
                ));
            }
        }

        camera.position = glm::vec3(0.0, 2.5, 8.5);
        camera.pitch = 0.0;
    }
}

fn main() {
    driver::run::<Pbr>("nightshade-renderer: PBR");
}
