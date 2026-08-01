use crate::commands::spawn_with_material;
use crate::components::Orbit;
use crate::data::{ARENA_SIZE, COLORS, NUM_LIGHTS, NUM_PILLARS_PER_SIDE};
use crate::queries::light_position;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;
use nightshade_render::contract::{Light, LightType};
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn initialize(
    mut commands: Commands,
    mut settings: ResMut<RenderSettings>,
    mut debug: ResMut<DebugDraw>,
    mut active_camera: ResMut<ActiveCamera>,
    mut materials: ResMut<MaterialRegistry>,
) {
    settings.atmosphere = Atmosphere::Space;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 1.0;
    debug.show_grid = false;

    let camera = spawn_pan_orbit_camera_at(
        &mut commands,
        Vec3::new(0.0, 4.0, 0.0),
        110.0,
        0.8,
        0.5,
        "Main Camera".to_string(),
    );
    active_camera.0 = Some(camera);

    let commands = &mut commands;
    let materials = &mut *materials;

    spawn_with_material(
        commands,
        materials,
        "Cube",
        Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(ARENA_SIZE * 2.0, 1.0, ARENA_SIZE * 2.0),
        Material {
            base_color: [0.15, 0.15, 0.18, 1.0],
            roughness: 0.8,
            metallic: 0.0,
            ..Default::default()
        },
    );

    let spacing = (ARENA_SIZE * 2.0) / (NUM_PILLARS_PER_SIDE as f32 + 1.0);
    let start = -ARENA_SIZE + spacing;
    for row in 0..NUM_PILLARS_PER_SIDE {
        for column in 0..NUM_PILLARS_PER_SIDE {
            let height = 4.0 + ((row + column) as f32 * 0.5) % 4.0;
            spawn_with_material(
                commands,
                materials,
                "Cube",
                Vec3::new(
                    start + column as f32 * spacing,
                    height / 2.0,
                    start + row as f32 * spacing,
                ),
                Vec3::new(1.0, height, 1.0),
                Material {
                    base_color: [0.4, 0.35, 0.3, 1.0],
                    roughness: 0.6,
                    metallic: 0.1,
                    ..Default::default()
                },
            );
        }
    }

    for index in 0..NUM_LIGHTS {
        let color = COLORS[index % COLORS.len()];
        let position = light_position(index, 0.0);

        let light = commands.spawn((
            Name(format!("Light_{index}")),
            LocalTransform {
                translation: position,
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            GlobalTransform::default(),
            Light {
                light_type: LightType::Point,
                color,
                intensity: 3.0,
                range: 10.0,
                inner_cone_angle: 0.0,
                outer_cone_angle: 0.0,
                cast_shadows: false,
                shadow_bias: 0.007,
                ..Default::default()
            },
        ));
        commands.set(light, Orbit(index));

        let sphere = spawn_with_material(
            commands,
            materials,
            "Sphere",
            position,
            Vec3::new(0.2, 0.2, 0.2),
            Material {
                base_color: [0.0, 0.0, 0.0, 1.0],
                emissive_factor: [color.x * 3.0, color.y * 3.0, color.z * 3.0],
                unlit: true,
                ..Default::default()
            },
        );
        commands.set(sphere, Orbit(index));
    }
}
