use crate::components::{Grounded, Home, Phase, Speed};
use crate::data::{
    BEACON_RADIUS, BEACONS, Beacon, PLAYER_MASS, PLAYER_RADIUS, ROPE_THICKNESS, Rope, Runner,
    SLAB_TILE_SIZE, SLABS, START,
};
use crate::systems::textures;
use crate::theme::{BEACON_COLOR, BEACON_GLOW, ROPE_COLOR, ROPE_GLOW, RUNNER_COLOR};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_physics::commands::spawn_dynamic_physics_sphere_with_material;
use nightshade_physics::commands::spawn_static_physics_cube_with_material;
use nightshade_physics::contract::components::{ColliderComponent, RigidBodyComponent};
use nightshade_physics::contract::types::LockedAxes;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_scene::prelude::{LocalTransform, Visibility};

pub fn build_canyon(commands: &mut Commands) {
    for slab in SLABS.iter() {
        let size = Vec3::new(slab.size[0], slab.size[1], slab.size[2]);
        let tiling = [
            (size.x / SLAB_TILE_SIZE).max(1.0),
            (size.z.max(size.y) / SLAB_TILE_SIZE).max(1.0),
        ];
        spawn_static_physics_cube_with_material(
            commands,
            Vec3::new(slab.center[0], slab.center[1], slab.center[2]),
            size,
            textures::tiled_material(slab.texture, tiling),
        );
    }
}

pub fn spawn_runner(commands: &mut Commands) -> Entity {
    let start = Vec3::new(START[0], START[1], START[2]);
    let entity = spawn_dynamic_physics_sphere_with_material(
        commands,
        start,
        PLAYER_RADIUS,
        PLAYER_MASS,
        textures::solid_material(RUNNER_COLOR, 0.5, 0.0),
    );
    commands.queue(move |storage, _| {
        storage.set(
            entity,
            RigidBodyComponent {
                locked_axes: LockedAxes::ROTATION_LOCKED,
                ..RigidBodyComponent::new_dynamic()
                    .with_translation(start.x, start.y, start.z)
                    .with_mass(PLAYER_MASS)
                    .with_ccd(true)
            },
        );
        storage.set(
            entity,
            ColliderComponent::new_ball(PLAYER_RADIUS)
                .with_friction(0.0)
                .with_restitution(0.0),
        );
        storage.set(entity, Visibility { visible: false });
    });
    commands.set(entity, Speed::default());
    commands.set(entity, Grounded::default());
    commands.add_tag_type::<Runner>(entity);
    entity
}

pub fn spawn_rope(registry: &mut MaterialRegistry, commands: &mut Commands) -> Entity {
    let entity = spawn_mesh_at(
        commands,
        "Cylinder",
        Vec3::zeros(),
        Vec3::new(ROPE_THICKNESS, 1.0, ROPE_THICKNESS),
    );
    let material = insert_material(
        registry,
        "tether_rope",
        textures::glowing_material(ROPE_COLOR, ROPE_GLOW),
    );
    commands.set(entity, MaterialRef::new(material));
    commands.queue(move |storage, _| {
        storage.set(entity, Visibility { visible: false });
    });
    commands.add_tag_type::<Rope>(entity);
    entity
}

pub fn spawn_beacons(registry: &mut MaterialRegistry, commands: &mut Commands) {
    let material = insert_material(
        registry,
        "tether_beacon",
        textures::glowing_material(BEACON_COLOR, BEACON_GLOW),
    );
    for (index, spot) in BEACONS.iter().enumerate() {
        let home = Vec3::new(spot[0], spot[1], spot[2]);
        let entity = spawn_mesh_at(commands, "Torus", home, Vec3::repeat(BEACON_RADIUS));
        commands.set(entity, MaterialRef::new(material));
        commands.set(entity, Home(home));
        commands.set(entity, Phase(index as f32 * 0.9));
        if let Some(transform) = commands.staged_mut::<LocalTransform>(entity) {
            transform.rotation =
                nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x());
        }
        commands.restage::<LocalTransform>(entity);
        commands.add_tag_type::<Beacon>(entity);
    }
}
