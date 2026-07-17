//! A directional sun casting cascaded shadow maps across a floor of mixed
//! primitives.
//!
//! ```sh
//! cargo run --example shadows --features wgpu
//! ```

use nalgebra_glm as glm;
use nightshade_renderer::config::Atmosphere;
use nightshade_renderer::material::Material;
use nightshade_renderer::procedural_meshes::{
    create_cube_mesh, create_plane_mesh, create_sphere_mesh,
};

#[path = "common/driver.rs"]
mod driver;

use driver::{Frame, Scene};

fn setup(scene: &mut Scene) {
    scene.settings.atmosphere = Atmosphere::Sky;
    scene.settings.show_sky = true;
    scene.settings.ambient_light = [0.3, 0.35, 0.45, 0.3];
    scene.sun(glm::vec3(-0.6, -1.0, -0.4), glm::vec3(1.0, 0.96, 0.88), 4.0);

    let floor = scene.material(Material {
        base_color: [0.72, 0.72, 0.74, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    scene.mesh("floor", create_plane_mesh(50.0));
    scene.object("floor", floor, glm::translation(&glm::vec3(0.0, 0.0, 0.0)));

    scene.mesh("cube", create_cube_mesh());
    scene.mesh("sphere", create_sphere_mesh(0.6, 24));
    let material = scene.material(Material {
        base_color: [0.85, 0.3, 0.25, 1.0],
        roughness: 0.45,
        ..Default::default()
    });
    for index in -3..=3 {
        let x = index as f32 * 2.2;
        if index % 2 == 0 {
            let transform = glm::translation(&glm::vec3(x, 0.7, 0.0))
                * glm::rotation(0.4, &glm::vec3(0.0, 1.0, 0.0));
            scene.object("cube", material, transform);
        } else {
            scene.object(
                "sphere",
                material,
                glm::translation(&glm::vec3(x, 0.7, 0.0)),
            );
        }
    }

    scene.camera.position = glm::vec3(0.0, 4.5, 12.0);
    scene.camera.pitch = -0.3;
}

fn update(_scene: &mut Scene, _state: &mut (), _frame: &Frame) {}

fn main() {
    driver::run("nightshade-renderer: shadows", setup, update);
}
