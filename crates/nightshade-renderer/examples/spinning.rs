//! A field of spinning colored primitives, each rotating about its own axis.
//! The scene lives entirely in the renderer's own ECS world.
//!
//! ```sh
//! cargo run --example spinning --features wgpu,hdr
//! ```

use freecs::Entity;
use nalgebra_glm as glm;
use nightshade_renderer::config::Atmosphere;
use nightshade_renderer::material::Material;
use nightshade_renderer::procedural_meshes::{
    create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_sphere_mesh, create_torus_mesh,
};

#[path = "common/driver.rs"]
mod driver;

use driver::{Frame, Scene};

const GRID: i32 = 8;

struct Spinner {
    entity: Entity,
    base: glm::Vec3,
    axis: glm::Vec3,
    speed: f32,
}

struct Spinners(Vec<Spinner>);

fn setup(scene: &mut Scene) -> Spinners {
    scene.settings.atmosphere = Atmosphere::Sky;
    scene.settings.show_sky = true;
    scene.settings.ambient_light = [0.4, 0.45, 0.55, 1.0];
    scene.sun(glm::vec3(-0.5, -0.9, -0.5), glm::vec3(1.0, 0.96, 0.9), 3.0);

    scene.mesh("cube", create_cube_mesh());
    scene.mesh("sphere", create_sphere_mesh(0.5, 32));
    scene.mesh("cylinder", create_cylinder_mesh(0.4, 1.0, 24));
    scene.mesh("cone", create_cone_mesh(0.5, 1.0, 24));
    scene.mesh("torus", create_torus_mesh(0.45, 0.18, 24, 16));
    let meshes = ["cube", "sphere", "cylinder", "cone", "torus"];

    let spinners = Spinners(
        (0..GRID * GRID)
            .map(|index| {
                let (x, z) = (index / GRID, index % GRID);
                let (u, v) = (x as f32 / (GRID - 1) as f32, z as f32 / (GRID - 1) as f32);
                let material = scene.material(Material {
                    base_color: [
                        0.35 + 0.6 * u,
                        0.35 + 0.6 * v,
                        0.35 + 0.6 * (1.0 - u * v),
                        1.0,
                    ],
                    metallic: 0.2,
                    roughness: 0.4,
                    ..Default::default()
                });
                let base = glm::vec3(
                    (x - GRID / 2) as f32 * 1.8,
                    1.4,
                    (z - GRID / 2) as f32 * 1.8,
                );
                let entity = scene.object(
                    meshes[index as usize % meshes.len()],
                    material,
                    glm::translation(&base),
                );
                let axis = glm::normalize(&glm::vec3(
                    (index % 5) as f32 - 2.0,
                    3.0,
                    (index % 3) as f32 - 1.0,
                ));
                Spinner {
                    entity,
                    base,
                    axis,
                    speed: 0.6 + (index % 5) as f32 * 0.3,
                }
            })
            .collect(),
    );

    scene.camera.position = glm::vec3(0.0, 9.0, 17.0);
    scene.camera.pitch = -0.35;
    spinners
}

fn update(scene: &mut Scene, spinners: &mut Spinners, frame: &Frame) {
    for spinner in spinners.0.iter() {
        let transform = glm::translation(&spinner.base)
            * glm::rotation(frame.time * spinner.speed, &spinner.axis);
        scene.set_transform(spinner.entity, transform);
    }
}

fn main() {
    driver::run("nightshade-renderer: spinning primitives", setup, update);
}
