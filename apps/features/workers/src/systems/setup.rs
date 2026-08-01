use crate::commands::{keep_the_shapes, spawn_a_marker, spawn_a_stand, spawn_the_floor};
use crate::resources::{Gallery, Panel};
use crate::systems::panel;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::data::material::Material;
use nightshade_render::prelude::MeshCache;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn initialize(
    mut settings: ResMut<RenderSettings>,
    mut active_camera: ResMut<ActiveCamera>,
    mut registry: ResMut<MaterialRegistry>,
    mut mesh_cache: ResMut<MeshCache>,
    gallery: Res<Gallery>,
    mut panel: ResMut<Panel>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::Sky;
    settings.bloom_enabled = true;

    let stone = [
        (theme::FLOOR_MATERIAL, theme::FLOOR_COLOR),
        (theme::STAND_MATERIAL, theme::STAND_COLOR),
        (theme::MARKER_MATERIAL, theme::MARKER_COLOR),
        (theme::STATUE_MATERIAL, theme::STATUE_COLOR),
    ];
    for (name, color) in stone {
        insert_material(
            &mut registry,
            name,
            Material {
                base_color: color,
                roughness: theme::ROUGH,
                metallic: 0.0,
                ..Default::default()
            },
        );
    }

    let sun = spawn_sun(&mut commands);
    commands.queue(move |storage, _| {
        if let Some(light) = storage.get_mut::<nightshade_render::contract::Light>(sun) {
            light.intensity = theme::SUN_INTENSITY;
            light.cast_shadows = true;
        }
    });

    let camera = spawn_pan_orbit_camera_at(
        &mut commands,
        Vec3::new(0.0, theme::CAMERA_HEIGHT, 0.0),
        theme::CAMERA_BACK,
        theme::CAMERA_YAW,
        theme::CAMERA_PITCH,
        "Camera".to_string(),
    );
    active_camera.0 = Some(camera);

    keep_the_shapes(&mut mesh_cache);
    spawn_the_floor(&mut commands);
    for slot in 0..theme::STANDS {
        spawn_a_stand(&mut commands, slot);
        spawn_a_marker(&mut commands, slot);
    }

    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    *panel = panel::build(&mut tree, &gallery);
    tree.finish();
}
