use nalgebra_glm::{Quat, Vec3};
use nightshade::ecs::{Entity, Storage};
use nightshade_camera::components::{Camera, PerspectiveCamera, Projection};
use nightshade_graphics::commands::builtins::despawn_entities_now;
use nightshade_render::commands::material_registry::{insert_material, material_registry_release};
use nightshade_render::commands::mesh_cache::{
    mesh_cache_insert, mesh_cache_protect, mesh_cache_release,
};
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
use nightshade_render::components::culling::CastsShadow;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::bounding_volume::{BoundingVolume, OrientedBoundingBox};
use nightshade_render::data::config::TonemapAlgorithm;
use nightshade_render::data::geometry::Mesh;
use nightshade_render::data::material::Material;
use nightshade_scene::contract::primitives::{Name, Visibility};
use nightshade_scene::contract::{GlobalTransform, LocalTransform};

use crate::components::{DemoOwned, Distance, DistanceLimits, Focus, Pitch, Yaw};
use crate::resources::DemoScene;

pub fn begin(scene: &mut DemoScene<'_>, demo: &'static str, first: bool) {
    if first {
        let settings = scene.settings.clone();
        let camera = scene.active_camera.0;
        let runtime = &mut *scene.runtime;
        runtime.saved_settings = Some(settings);
        runtime.saved_camera = camera;

        let settings = &mut *scene.settings;
        settings.color_grading.auto_exposure = false;
        settings.color_grading.exposure = 1.0;
        settings.color_grading.exposure_compensation_ev = 0.0;
        settings.color_grading.tonemap_algorithm = TonemapAlgorithm::None;
    }
    let runtime = &mut *scene.runtime;
    runtime.mounting = Some(demo);
    runtime.mounting_bit = 1 << (runtime.active.len() % 32);
}

pub fn end(storage: &mut Storage, scene: &mut DemoScene<'_>, demo: &'static str) {
    let owned: Vec<Entity> = storage
        .query_entities::<(DemoOwned,)>()
        .filter(|entity| {
            storage
                .get::<DemoOwned>(*entity)
                .is_some_and(|owner| owner.0 == demo)
        })
        .collect();
    if !owned.is_empty() {
        despawn_owned(storage, scene, &owned);
    }

    let runtime = &mut *scene.runtime;
    let meshes = runtime.meshes.remove(demo).unwrap_or_default();
    let materials = runtime.materials.remove(demo).unwrap_or_default();
    runtime.cameras.remove(demo);
    runtime.active.retain(|held| *held != demo);

    let mesh_cache = &mut *scene.meshes;
    for name in &meshes {
        mesh_cache_release(mesh_cache, name);
    }
    let material_registry = &mut *scene.materials;
    for name in &materials {
        material_registry_release(material_registry, name);
    }
}

pub fn rest(scene: &mut DemoScene<'_>) {
    let runtime = &mut *scene.runtime;
    let settings = runtime.saved_settings.take();
    let camera = runtime.saved_camera.take();
    runtime.mounting = None;

    if let Some(settings) = settings {
        *scene.settings = settings;
    }
    scene.active_camera.0 = camera;
}

pub fn despawn_owned(storage: &mut Storage, scene: &mut DemoScene<'_>, targets: &[Entity]) {
    let mut caches = nightshade_graphics::commands::builtins::EntityCaches {
        entities: scene.entities,
        meshes: scene.meshes,
        mesh_state: scene.mesh_state,
        textures: scene.textures,
        materials: scene.materials,
        hierarchy: scene.hierarchy,
    };
    despawn_entities_now(storage, &mut caches, targets);
}

pub fn spawn_owned<B: nightshade::ecs::Bundle>(
    storage: &mut Storage,
    scene: &mut DemoScene<'_>,
    bundle: B,
) -> Entity {
    let runtime = &*scene.runtime;
    let owner = runtime.mounting.unwrap_or("");
    let bit = runtime.mounting_bit.max(1);
    let entity = storage.spawn(bundle);
    storage.set(entity, DemoOwned(owner));
    storage.set(entity, nightshade_render::components::CullingMask(bit));
    entity
}

pub fn register_mesh(scene: &mut DemoScene<'_>, name: &str, mesh: Mesh) {
    let cache = &mut *scene.meshes;
    mesh_cache_insert(cache, name.to_string(), with_bounds(mesh));
    mesh_cache_protect(cache, name.to_string());

    let runtime = &mut *scene.runtime;
    let owner = runtime.mounting.unwrap_or("");
    let held = runtime.meshes.entry(owner).or_default();
    if !held.iter().any(|name_held| name_held == name) {
        held.push(name.to_string());
    }
}

fn with_bounds(mesh: Mesh) -> Mesh {
    let mut lowest = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut highest = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    for vertex in &mesh.vertices {
        let position = Vec3::new(vertex.position[0], vertex.position[1], vertex.position[2]);
        lowest = lowest.inf(&position);
        highest = highest.sup(&position);
    }
    if mesh.vertices.is_empty() {
        return mesh;
    }
    let bounds = BoundingVolume::new(
        OrientedBoundingBox::from_aabb(lowest, highest),
        (highest - lowest).norm() * 0.5,
    );
    Mesh {
        bounding_volume: Some(bounds),
        ..mesh
    }
}

pub fn register_material(scene: &mut DemoScene<'_>, name: &str, material: Material) {
    let registry = &mut *scene.materials;
    insert_material(registry, name.to_string(), material);

    let runtime = &mut *scene.runtime;
    let owner = runtime.mounting.unwrap_or("");
    let held = runtime.materials.entry(owner).or_default();
    if !held.iter().any(|name_held| name_held == name) {
        held.push(name.to_string());
    }
}

pub fn spawn_owned_mesh(
    storage: &mut Storage,
    scene: &mut DemoScene<'_>,
    mesh_name: &str,
    material_name: &str,
    transform: LocalTransform,
) -> Entity {
    let material_id = nightshade_render::queries::material_registry::material_registry_resolve(
        scene.materials,
        material_name,
    )
    .unwrap_or_default();
    let entity = spawn_owned(
        storage,
        scene,
        (
            Name(mesh_name.to_string()),
            transform,
            GlobalTransform(transform.as_matrix()),
            RenderMesh::new(mesh_name),
            MaterialRef::new(material_id),
            CastsShadow,
            Visibility { visible: true },
        ),
    );
    mesh_state_mark_entity_added(scene.mesh_state, entity);
    entity
}

pub fn spawn_owned_camera(
    storage: &mut Storage,
    scene: &mut DemoScene<'_>,
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
) -> Entity {
    let (position, rotation) = orbit_transform(focus, yaw, pitch, distance);
    let transform = LocalTransform {
        translation: position,
        rotation,
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    let entity = spawn_owned(
        storage,
        scene,
        (
            Name("Demo Camera".to_string()),
            transform,
            GlobalTransform(transform.as_matrix()),
            Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 45.0_f32.to_radians(),
                    z_far: None,
                    z_near: 0.01,
                }),
                smoothing: None,
            },
        ),
    );
    storage.set(entity, Focus(focus));
    storage.set(entity, Yaw(yaw));
    storage.set(entity, Pitch(pitch));
    storage.set(entity, Distance(distance));
    storage.set(entity, DistanceLimits::default());
    let runtime = &mut *scene.runtime;
    let bit = runtime.mounting_bit.max(1);
    if let Some(owner) = runtime.mounting {
        runtime.cameras.insert(owner, entity);
    }
    storage.set(
        entity,
        nightshade_render::components::CameraCullingMask(bit),
    );
    scene.active_camera.0 = Some(entity);
    entity
}

pub fn spawn_owned_sun(
    storage: &mut Storage,
    scene: &mut DemoScene<'_>,
    direction: Vec3,
    intensity: f32,
) -> Entity {
    let rotation = orbit_rotation(direction);
    let transform = LocalTransform {
        translation: -direction * 12.0,
        rotation,
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    spawn_owned(
        storage,
        scene,
        (
            Name("Demo Sun".to_string()),
            transform,
            GlobalTransform(transform.as_matrix()),
            nightshade_render::components::Light {
                light_type: nightshade_render::components::LightType::Directional,
                color: Vec3::new(1.0, 0.96, 0.9),
                intensity,
                range: 100.0,
                inner_cone_angle: std::f32::consts::FRAC_PI_6,
                outer_cone_angle: std::f32::consts::FRAC_PI_4,
                cast_shadows: false,
                ..Default::default()
            },
            Visibility { visible: true },
        ),
    )
}

fn orbit_transform(focus: Vec3, yaw: f32, pitch: f32, distance: f32) -> (Vec3, Quat) {
    let yaw_rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
    let pitch_rotation = nalgebra_glm::quat_angle_axis(-pitch, &Vec3::x());
    let rotation = yaw_rotation * pitch_rotation;
    let offset = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, distance));
    (focus + offset, rotation)
}

pub fn orbit_rotation(direction: Vec3) -> Quat {
    let direction = if direction.magnitude() > f32::EPSILON {
        direction.normalize()
    } else {
        -Vec3::z()
    };
    let yaw = (-direction.x).atan2(-direction.z);
    let pitch = direction.y.asin();
    nalgebra_glm::quat_angle_axis(yaw, &Vec3::y())
        * nalgebra_glm::quat_angle_axis(pitch, &Vec3::x())
}
