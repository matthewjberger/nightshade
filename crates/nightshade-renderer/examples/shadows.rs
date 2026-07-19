//! A directional sun casting shadows across a floor of mixed primitives. Shows
//! the `ShadowCaster` tag: the shapes are tagged so the shadow pass draws them
//! as occluders, while the untagged floor only receives.
//!
//! ```sh
//! cargo run --example shadows --features wgpu,hdr
//! ```

use nalgebra_glm as glm;
use nightshade_renderer::config::Atmosphere;
use nightshade_renderer::material::Material;
use nightshade_renderer::mesh_cache::mesh_cache_insert;
use nightshade_renderer::procedural_meshes::{
    create_cube_mesh, create_plane_mesh, create_sphere_mesh,
};
use nightshade_renderer::render_layer::RenderLayer;
use nightshade_renderer::render_world::{
    CullingMask, Layer, MaterialId, MeshName, MorphWeights, ShadowCaster, Transform, Visible,
};
use nightshade_renderer::wgpu::render_configs::SingleViewHost;

#[path = "common/driver.rs"]
mod driver;

use driver::{Camera, Example, directional_light};

struct Shadows;

impl Example for Shadows {
    type State = ();

    fn setup(host: &mut SingleViewHost, camera: &mut Camera) {
        host.settings.atmosphere = Atmosphere::Sky;
        host.settings.show_sky = true;
        host.settings.ambient_light = [0.3, 0.35, 0.45, 0.3];

        host.scene_world.spawn(directional_light(
            glm::vec3(-0.6, -1.0, -0.4),
            glm::vec3(1.0, 0.96, 0.88),
            4.0,
        ));

        mesh_cache_insert(
            &mut host.mesh_cache,
            "floor".to_string(),
            create_plane_mesh(50.0),
        );
        let floor = host.state.render_materials.add(Material {
            base_color: [0.72, 0.72, 0.74, 1.0],
            roughness: 0.9,
            ..Material::default()
        });
        host.scene_world.spawn((
            Transform(glm::translation(&glm::vec3(0.0, 0.0, 0.0))),
            MeshName("floor".to_string()),
            MaterialId(floor),
            Visible(true),
            MorphWeights([0.0; 8]),
            CullingMask(u32::MAX),
            Layer(RenderLayer::WORLD),
        ));

        mesh_cache_insert(&mut host.mesh_cache, "cube".to_string(), create_cube_mesh());
        mesh_cache_insert(
            &mut host.mesh_cache,
            "sphere".to_string(),
            create_sphere_mesh(0.6, 24),
        );
        let material = host.state.render_materials.add(Material {
            base_color: [0.85, 0.3, 0.25, 1.0],
            roughness: 0.45,
            ..Material::default()
        });
        for index in -3..=3 {
            let x = index as f32 * 2.2;
            let (mesh, transform) = if index % 2 == 0 {
                (
                    "cube",
                    glm::translation(&glm::vec3(x, 0.7, 0.0))
                        * glm::rotation(0.4, &glm::vec3(0.0, 1.0, 0.0)),
                )
            } else {
                ("sphere", glm::translation(&glm::vec3(x, 0.7, 0.0)))
            };
            let entity = host.scene_world.spawn((
                Transform(transform),
                MeshName(mesh.to_string()),
                MaterialId(material),
                Visible(true),
                MorphWeights([0.0; 8]),
                CullingMask(u32::MAX),
                Layer(RenderLayer::WORLD),
            ));
            host.scene_world.add_tag_type::<ShadowCaster>(entity);
        }

        camera.position = glm::vec3(0.0, 4.5, 12.0);
        camera.pitch = -0.3;
    }
}

fn main() {
    driver::run::<Shadows>("nightshade-renderer: shadows");
}
