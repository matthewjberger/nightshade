use crate::resources::{Handles, Panel, Walk};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::commands::spawn_first_person_camera;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_gltf::queries::sample_model_parts as sample_model;
use nightshade_graphics::prelude::*;
use nightshade_physics::commands::spawn_character_body;
use nightshade_physics::contract::components::{
    CharacterControllerComponent, ColliderComponent, ColliderShape, RigidBodyComponent,
};
use nightshade_render::components::culling::CastsShadow;
use nightshade_render::contract::{Light, LightType};
use nightshade_render::data::config::Fog;
use nightshade_render::data::material::Material;
use nightshade_render::queries::lights::sun_rotation;

struct Stone {
    vertices: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
}

pub fn initialize(
    mut render_settings: ResMut<RenderSettings>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut handles: ResMut<Handles>,
    mut active_camera: ResMut<ActiveCamera>,
    mut panel_state: ResMut<Panel>,
    mut walk_state: ResMut<Walk>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    settle_the_look(&mut render_settings);
    let handle = nightshade_load::load(
        &mut loading,
        nightshade_load::Source::Path(sample_model(theme::MODEL)),
    );
    handles.model = Some(handle);
    let look = render_settings.clone();
    let (orbit, body, eyes, sun, lanterns) = {
        let context = build.context();
        let commands = &mut *context.commands;
        {
            let orbit = spawn_pan_orbit_camera_at(
                commands,
                theme::CAMERA_TARGET,
                theme::CAMERA_DISTANCE,
                theme::CAMERA_YAW,
                theme::CAMERA_PITCH,
                "Camera".to_string(),
            );
            let sun = commands.spawn((
                Name("Sun".to_string()),
                LocalTransform {
                    translation: Vec3::new(0.0, 30.0, 0.0),
                    rotation: sun_rotation(theme::SUN_YAW, theme::SUN_PITCH),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
                GlobalTransform::default(),
                Light {
                    light_type: LightType::Directional,
                    color: theme::SUN_COLOR,
                    intensity: theme::SUN_INTENSITY,
                    cast_shadows: true,
                    shadow_bias: 0.02,
                    ..Default::default()
                },
            ));

            let mut lanterns = [None; 3];
            for (index, at) in theme::LANTERNS.iter().enumerate() {
                lanterns[index] = Some(commands.spawn((
                    Name(format!("Lantern {index}")),
                    LocalTransform {
                        translation: *at,
                        rotation: Quat::identity(),
                        scale: Vec3::new(1.0, 1.0, 1.0),
                    },
                    GlobalTransform::default(),
                    Light {
                        light_type: LightType::Point,
                        color: theme::LANTERN_COLOR,
                        intensity: theme::LANTERN_INTENSITY,
                        range: theme::LANTERN_RANGE,
                        cast_shadows: true,
                        ..Default::default()
                    },
                )));
                let flame = spawn_mesh_at(commands, "Sphere", *at, Vec3::new(0.12, 0.12, 0.12));
                commands.remove::<(CastsShadow,)>(flame);
                queue_set_material(
                    commands,
                    flame,
                    format!("coat {}", flame.id),
                    Material {
                        base_color: [0.0, 0.0, 0.0, 1.0],
                        emissive_factor: [
                            theme::LANTERN_COLOR.x * 6.0,
                            theme::LANTERN_COLOR.y * 6.0,
                            theme::LANTERN_COLOR.z * 6.0,
                        ],
                        unlit: true,
                        ..Default::default()
                    },
                );
            }

            let body = spawn_character_body(commands, theme::WALK_START, "Walker");
            let eyes = spawn_first_person_camera(
                commands,
                body,
                Vec3::new(0.0, theme::EYE_HEIGHT, 0.0),
                theme::EYE_FIELD_OF_VIEW,
            );
            commands.queue(move |storage, _| {
                if let Some(controller) = storage.get_mut::<CharacterControllerComponent>(body) {
                    controller.engine_input_enabled = false;
                    controller.max_speed = theme::WALK_SPEED;
                }
            });

            (orbit, body, eyes, sun, lanterns)
        }
    };
    let panel = crate::systems::panel::build(build.context(), &look);
    active_camera.0 = Some(orbit);
    let opening = *panel_state;
    *panel_state = Panel {
        wanted: opening.wanted,
        rate_in_title_on: opening.rate_in_title_on,
        ..panel
    };
    handles.sun = Some(sun);
    handles.lanterns = lanterns;
    walk_state.orbit = Some(orbit);
    walk_state.body = Some(body);
    walk_state.eyes = Some(eyes);
}

fn settle_the_look(settings: &mut RenderSettings) {
    settings.atmosphere = Atmosphere::Sunset;
    settings.fog = Some(Fog {
        color: theme::FOG_COLOR,
        start: 0.0,
        end: theme::FOG_END,
        volumetric: true,
        density: theme::FOG_DENSITY,
        anisotropy: theme::FOG_ANISOTROPY,
        height: theme::FOG_HEIGHT,
        height_falloff: theme::FOG_HEIGHT_FALLOFF,
        ..Fog::default()
    });
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.35;
    settings.anti_aliasing = AntiAliasing::Taa;
    settings.ssao_enabled = true;
    settings.ssao_traced = false;
    settings.ssr_enabled = false;
    settings.ssr_traced = false;
    settings.ssgi_enabled = false;
    settings.sun_shadow_traced = false;
    settings.color_grading.auto_exposure = true;
    settings.color_grading.auto_exposure_min_ev = theme::EXPOSURE_LEAST_STOPS;
    settings.color_grading.auto_exposure_max_ev = theme::EXPOSURE_MOST_STOPS;
    settings.depth_of_field.enabled = false;
    settings.depth_of_field.max_blur_radius = theme::FOCUS_BLUR_RADIUS;
    settings.taa_sharpness = theme::SHARPEN;
}

pub fn lay_the_floor(
    mut handles: ResMut<Handles>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut window: ResMut<nightshade_platform::prelude::Window>,
    mut commands: Commands,
    parts: nightshade_prefab::PrefabParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(handle) = handles.model else {
        return;
    };
    let Some(outcome) = nightshade_load::take(&mut loading, handle) else {
        return;
    };
    handles.model = None;
    let Some(stone) = bring_in_the_model(storage, parts, &mut window, outcome) else {
        return;
    };
    commands.spawn((
        Name("Floor".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        RigidBodyComponent::new_static(),
        ColliderComponent {
            shape: ColliderShape::TriMesh {
                vertices: stone.vertices,
                indices: stone.indices,
            },
            ..Default::default()
        },
    ));
}

fn bring_in_the_model(
    storage: &mut nightshade::ecs::Storage,
    parts: nightshade_prefab::PrefabParts<'_>,
    window: &mut nightshade_platform::prelude::Window,
    outcome: Result<nightshade_prefab::data::ModelImport, String>,
) -> Option<Stone> {
    let mut import = match outcome {
        Ok(import) => import,
        Err(error) => {
            tracing::error!("{} did not load: {error}", theme::MODEL);
            window.should_exit = true;
            return None;
        }
    };
    let prefab = import.prefabs.first().cloned()?;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for node in &prefab.root_nodes {
        gather_stone(
            node,
            &Mat4::identity(),
            &import.meshes,
            &mut vertices,
            &mut indices,
        );
    }
    let mut spawn = nightshade_prefab::PrefabSpawn { storage, parts };
    nightshade_prefab::queue_model_load(&mut spawn, &mut import);
    nightshade_prefab::spawn_prefab(&mut spawn, &prefab, Vec3::zeros());
    (!indices.is_empty()).then_some(Stone { vertices, indices })
}

fn gather_stone(
    node: &nightshade_prefab::contract::PrefabNode,
    above: &Mat4,
    meshes: &std::collections::HashMap<String, nightshade_render::data::geometry::Mesh>,
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<[u32; 3]>,
) {
    let here = above * node.local_transform.as_matrix();
    if let Some(render_mesh) = node.components.render_mesh.as_ref()
        && let Some(mesh) = meshes.get(&render_mesh.name)
    {
        let first = vertices.len() as u32;
        for vertex in &mesh.vertices {
            let stood = here
                * nalgebra_glm::vec4(
                    vertex.position[0],
                    vertex.position[1],
                    vertex.position[2],
                    1.0,
                );
            vertices.push([stood.x, stood.y, stood.z]);
        }
        for corners in mesh.indices.as_chunks::<3>().0 {
            indices.push([first + corners[0], first + corners[1], first + corners[2]]);
        }
    }
    for child in &node.children {
        gather_stone(child, &here, meshes, vertices, indices);
    }
}
