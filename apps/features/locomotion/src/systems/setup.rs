use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::commands::spawn_third_person_camera;
use nightshade_graphics::prelude::*;
use nightshade_render::components::light::{Light, LightType};

use crate::data;
use crate::resources::{LocomotionResources, Screen};
use crate::systems::screens;

pub fn initialize(
    mut locomotion: ResMut<LocomotionResources>,
    mut settings: ResMut<RenderSettings>,
    mut registry: ResMut<nightshade_render::resources::material_registry::MaterialRegistry>,
    mut active_camera: ResMut<ActiveCamera>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    style: Res<nightshade_ui_menu::prelude::MenuStyle>,
    mut menus: ResMut<nightshade_ui_menu::prelude::Menus<Screen>>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::Sky;

    let sun = spawn_light_entity(&mut commands, Vec3::new(12.0, 18.0, 8.0), "Sun");
    commands.set(
        sun,
        Light {
            light_type: LightType::Directional,
            color: Vec3::new(1.0, 0.97, 0.9),
            intensity: 4.0,
            cast_shadows: true,
            ..Default::default()
        },
    );

    for step in 0..data::PILLAR_COUNT {
        let angle = step as f32 / data::PILLAR_COUNT as f32 * std::f32::consts::TAU;
        let height = 1.6 + (step % 3) as f32 * 0.9;
        let position = Vec3::new(
            angle.cos() * data::PILLAR_RING_RADIUS,
            height * 0.5,
            angle.sin() * data::PILLAR_RING_RADIUS,
        );
        let pillar = spawn_mesh(
            &mut commands,
            "Cylinder",
            position,
            Vec3::new(0.6, height, 0.6),
        );
        set_material(
            &mut registry,
            &mut commands,
            pillar,
            format!("pillar_{step}"),
            Material {
                base_color: [0.55, 0.5, 0.46, 1.0],
                roughness: 0.75,
                ..Default::default()
            },
        );
    }

    let body = commands.spawn((
        Name("Character".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
    ));

    let camera = spawn_third_person_camera(
        &mut commands,
        body,
        data::CAMERA_DISTANCE,
        0.0,
        data::CAMERA_PITCH,
        "Follow Camera".to_string(),
    );
    active_camera.0 = Some(camera);

    locomotion.character.body = Some(body);
    locomotion.character.camera = Some(camera);

    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    locomotion.screens = screens::build(&mut tree, &style, &mut menus);
    locomotion.character.asked = Some(nightshade_load::load(
        &mut loading,
        nightshade_load::Source::Bytes(data::CHARACTERS[0].1.to_vec()),
    ));
}
