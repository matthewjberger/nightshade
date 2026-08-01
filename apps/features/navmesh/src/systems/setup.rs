use crate::commands::{fox, interface, world};
use crate::data::HDR_SKYBOX;
use crate::resources::{Behavior, FoxModel, Ground, Interface, Scene, Stage, Trees, Viewpoint};
use nightshade::prelude::*;
use nightshade_camera_controller::commands::*;
use nightshade_graphics::prelude::*;
use nightshade_navmesh::commands::set_navmesh_debug_draw;

pub fn initialize(
    mut viewpoint: ResMut<Viewpoint>,
    mut panel: ResMut<Interface>,
    mut trees: ResMut<Trees>,
    mut model: ResMut<FoxModel>,
    ground: Res<Ground>,
    behavior: Res<Behavior>,
    stage: Stage,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let mut owned = Scene {
        storage,
        commands: &mut commands,
        stage,
    };
    let scene = &mut owned;

    scene.stage.settings.atmosphere = Atmosphere::Hdr;
    scene.stage.debug_draw.show_grid = false;

    load_hdr_skybox(scene.stage.queues, HDR_SKYBOX.to_vec());

    let sun = spawn_sun(scene.commands);
    scene.commands.set(
        sun,
        nightshade_render::contract::Light {
            light_type: nightshade_render::contract::LightType::Directional,
            cast_shadows: true,
            intensity: 5.0,
            ..Default::default()
        },
    );

    let focus = nalgebra_glm::vec3(0.0, 2.0, 0.0);
    let camera = spawn_pan_orbit_camera_at(
        scene.commands,
        focus,
        50.0,
        0.0,
        0.6,
        "Main Camera".to_string(),
    );
    scene.stage.active_camera.0 = Some(camera);
    viewpoint.0 = Some(camera);

    world::spawn_terrain(scene, &ground);
    world::spawn_trees(scene, &ground, &mut trees);
    world::build_navmesh(scene, &ground, &trees);
    fox::ask_for_the_model(scene, &mut model);

    set_navmesh_debug_draw(scene.stage.navmesh, behavior.show_navmesh);

    *panel = interface::build_ui(scene, &behavior);
}
